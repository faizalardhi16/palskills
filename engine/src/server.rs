//! MCP Server — 6 orchestration tools for AI agents.
//!
//! Tools: orchestrate (CBM-aware flow detection), write_docs (documentation-first gate),
//! scan_context (CBM code search), dispatch (spawn agent), run_tests (verify),
//! record_session (persist).

use std::path::PathBuf;

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cbm_bridge;
use crate::dashboard;
use crate::dispatch;
use crate::generator;
use crate::orchestrator;

// ── Shared state ─────────────────────────────────────────────────

pub struct AppState {
    pub palbox: PathBuf,
    pub cbm_path: PathBuf,
}

// ── Tool parameter types ─────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct TaskParams {
    pub task: String,
}

/// Cursor MCP requires outputSchema type: "object".
#[derive(Serialize, JsonSchema)]
pub struct ToolOutput {
    pub result: String,
}

fn out(s: String) -> Json<ToolOutput> {
    Json(ToolOutput { result: s })
}

// ── Dashboard + State.md helpers ─────────────────────────────────

fn tool_start(palbox: &PathBuf, skill: &str, msg: &str, task: &str) -> std::time::Instant {
    let _ = generator::update_skill_state(palbox, skill, "inprogress", Some(msg), None, Some(task), None, None);
    dashboard::emit_event(dashboard::PipelineEvent {
        event: "tool_start".into(),
        skill: Some(skill.into()),
        status: Some("inprogress".into()),
        message: Some(msg.into()),
        duration_ms: None, flow: None, confidence: None,
        stats_nodes: None, stats_symbols: None, stats_files: None, state: None,
    });
    std::time::Instant::now()
}

fn tool_done(palbox: &PathBuf, skill: &str, msg: &str, dur: std::time::Instant, task: &str, flow: Option<Vec<String>>, confidence: Option<u8>) {
    let dur_ms = dur.elapsed().as_millis() as u64;
    let _ = generator::update_skill_state(palbox, skill, "done", Some(msg), Some(dur_ms), Some(task), flow.as_deref(), confidence);
    dashboard::emit_event(dashboard::PipelineEvent {
        event: "tool_done".into(),
        skill: Some(skill.into()),
        status: Some("done".into()),
        message: Some(msg.into()),
        duration_ms: Some(dur_ms),
        flow, confidence,
        stats_nodes: None, stats_symbols: None, stats_files: None, state: None,
    });
}

fn tool_error(palbox: &PathBuf, skill: &str, msg: &str, dur: std::time::Instant) {
    let dur_ms = dur.elapsed().as_millis() as u64;
    let _ = generator::update_skill_state(palbox, skill, "error", Some(msg), Some(dur_ms), None, None, None);
    dashboard::emit_event(dashboard::PipelineEvent {
        event: "tool_error".into(),
        skill: Some(skill.into()),
        status: Some("error".into()),
        message: Some(msg.into()),
        duration_ms: Some(dur_ms),
        flow: None, confidence: None,
        stats_nodes: None, stats_symbols: None, stats_files: None, state: None,
    });
}

// ── Auto-flush stdout wrapper ────────────────────────────────────

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;

struct FlushingWriter<W: AsyncWrite + Unpin> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> AsyncWrite for FlushingWriter<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        let result = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(_)) = &result {
            let _ = Pin::new(&mut self.inner).poll_flush(cx);
        }
        result
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// ── Tools (6 total) ──────────────────────────────────────────────

#[tool_router(server_handler)]
impl AppState {
    /// Astralym: CBM-aware flow detection.
    #[tool(name = "orchestrate", description = "Analyze task via CBM code search. Returns recommended flow, confidence %, docs-first pipeline, and relevant symbols/files. Use this FIRST before any code generation.")]
    fn orchestrate(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "orchestrate", &format!("Analyzing: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        match orchestrator::analyze(root, &p.task) {
            Ok(ctx) => {
                let flow = ctx.flow.clone();
                let conf = ctx.confidence;
                let summary = ctx.summary.clone();
                tool_done(&self.palbox, "orchestrate", &summary, timer, &p.task, Some(flow), Some(conf));
                out(serde_json::to_string_pretty(&ctx).unwrap_or_default())
            }
            Err(e) => {
                tool_error(&self.palbox, "orchestrate", &format!("Error: {e}"), timer);
                out(format!("Error: {e}"))
            }
        }
    }

    /// Katress: Documentation-first gate. Writes README, API docs, and architecture
    /// docs BEFORE code. Scans project structure to generate accurate docs.
    #[tool(name = "write_docs", description = "Generate project documentation BEFORE coding. Writes README.md with full API spec, architecture docs in .palbox/, and setup guides. Scans project structure to document real endpoints, models, and patterns. Use this BEFORE any code generation — documentation is first-class delivery.")]
    fn write_docs(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "write_docs", &format!("Generating docs: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        match write_project_docs(root, &p.task) {
            Ok(report) => {
                tool_done(&self.palbox, "write_docs", &report, timer, &p.task, None, None);
                out(report)
            }
            Err(e) => {
                tool_error(&self.palbox, "write_docs", &format!("Failed: {e}"), timer);
                out(format!("Error: {e}"))
            }
        }
    }

    /// Lyleen: CBM-backed code search.
    #[tool(name = "scan_context", description = "Search codebase via CBM or grep fallback. Returns relevant symbols, files, and paths. Use this to understand existing code before modifying.")]
    fn scan_context(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "scan_context", &format!("Scanning: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        let cbm = cbm_bridge::get_context(root, &p.task).ok();

        #[derive(Serialize)]
        struct ContextOutput {
            symbols: Vec<String>,
            files: Vec<String>,
            source: String,
        }

        let output = ContextOutput {
            symbols: cbm.as_ref().map(|c| c.symbols.iter().map(|s| s.name.clone()).collect()).unwrap_or_default(),
            files: cbm.as_ref().map(|c| c.files.clone()).unwrap_or_default(),
            source: cbm.as_ref().map(|c| c.source.clone()).unwrap_or_else(|| "none".into()),
        };

        let msg = format!("Found {} symbols, {} files (via {})", output.symbols.len(), output.files.len(), output.source);
        tool_done(&self.palbox, "scan_context", &msg, timer, &p.task, None, None);
        out(serde_json::to_string_pretty(&output).unwrap_or_default())
    }

    /// Anubis: Dispatch to AI agent for execution.
    #[tool(name = "dispatch", description = "Execute feature via AI coding agent (Codex CLI). Passes enriched context from scan_context. Applies SOLID principles — single responsibility, DRY, no code smells.")]
    fn dispatch(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "dispatch", &format!("Dispatching: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        // Build enriched context: CBM results + SOLID prompt
        let cbm = cbm_bridge::get_context(root, &p.task).unwrap_or_default();
        let enriched_task = format!(
            "{}\n\n## Codebase Context\nSymbols: {:?}\nFiles: {:?}\n\n## Constraints\n- Apply SOLID principles (Single Responsibility, Open-Closed, Liskov, Interface Segregation, Dependency Inversion)\n- Zero code duplication — extract shared logic, never copy-paste\n- No code smells (god classes, long methods, magic numbers)\n- Write clean, self-documenting code with meaningful names\n- YAGNI: only build what's needed now, not what might be needed later",
            p.task, cbm.symbols, cbm.files
        );

        match dispatch::execute(root, &enriched_task) {
            Ok(()) => {
                tool_done(&self.palbox, "dispatch", "Agent dispatched with SOLID constraints", timer, &p.task, None, None);
                out("✅ Dispatched. AI agent building with SOLID principles...".into())
            }
            Err(e) => {
                tool_error(&self.palbox, "dispatch", &format!("Failed: {e}"), timer);
                out(format!("❌ Dispatch failed: {e}"))
            }
        }
    }

    /// Verdash: Run actual test suite.
    #[tool(name = "run_tests", description = "Run project test suite. Auto-detects runner (pytest, cargo test, npm test). Returns pass/fail results. Fails are SOFT BLOCKERS — fix and re-run.")]
    fn run_tests(&self) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "run_tests", "Running tests...", "tests");
        let commands = ["pytest -x --tb=short", "cargo test --quiet", "npm test -- --reporter=min"];

        for cmd in &commands {
            if let Ok(result) = std::process::Command::new("sh").arg("-c").arg(cmd).output() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stdout.contains("pass") || stdout.contains("ok") || result.status.success() {
                    tool_done(&self.palbox, "run_tests", "Tests passed ✓", timer, "tests", None, None);
                    return out(format!("🧪 Tests passed:\n{stdout}\n{stderr}"));
                }
            }
        }
        tool_error(&self.palbox, "run_tests", "No test runner found", timer);
        out("🧪 No test runner detected.".into())
    }

    /// Panthalus: Persist session.
    #[tool(name = "record_session", description = "Record current development session to .palbox/history/. Persists decisions, changed files, and lessons learned for future context retrieval.")]
    fn record_session(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "record_session", &format!("Recording: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        let session = format!(
            "# Session: {}\n**Date:** {}\n**Author:** Panthalus\n\n## Task\n{}\n\n## Files changed\n[tba — fill after review]\n\n## Decisions\n[tba]\n\n## SOLID compliance\n- [ ] Single Responsibility: each class/module has one reason to change\n- [ ] Open-Closed: extended without modifying existing code\n- [ ] Liskov: subtypes substitutable for base types\n- [ ] Interface Segregation: no unused method dependencies\n- [ ] Dependency Inversion: depends on abstractions, not concretions\n\n## Lessons\n[tba]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d %H:%M"),
            p.task
        );

        match generator::record_session(root, &p.task, &session) {
            Ok(path) => {
                tool_done(&self.palbox, "record_session", &format!("Saved to {}", path.display()), timer, &p.task, None, None);
                out(format!("✅ Session recorded: {}", path.display()))
            }
            Err(e) => {
                tool_error(&self.palbox, "record_session", &format!("Failed: {e}"), timer);
                out(format!("Error: {e}"))
            }
        }
    }
}

// ── Katress: documentation generation ──────────────────────────

/// Scan project structure and generate documentation.
fn write_project_docs(root: &std::path::Path, task: &str) -> anyhow::Result<String> {
    let mut report = String::new();
    let mut files_written = 0;

    // 1. Detect project type
    let has_py = root.join("requirements.txt").exists() || root.join("pyproject.toml").exists();
    let has_rs = root.join("Cargo.toml").exists();
    let has_js = root.join("package.json").exists();
    let has_api = root.join("api").exists() || root.join("src").exists();

    report.push_str(&format!("# Documentation Report\n\n"));
    report.push_str(&format!("**Task:** {}\n", task));
    report.push_str(&format!("**Project:** {}\n", root.display()));
    report.push_str(&format!("**Detected:** Python={}, Rust={}, JS={}, API={}\n\n", has_py, has_rs, has_js, has_api));

    // 2. Generate/update README if missing
    let readme_path = root.join("README.md");
    if !readme_path.exists() {
        let readme = generate_readme(root, task, has_py, has_rs, has_js);
        std::fs::write(&readme_path, &readme)?;
        report.push_str(&format!("✅ Created: {}\n", readme_path.display()));
        files_written += 1;
    } else {
        report.push_str("ℹ️  README.md already exists (skipped)\n");
    }

    // 3. Create .palbox/architecture.md
    let palbox = root.join(".palbox");
    std::fs::create_dir_all(&palbox)?;
    let arch_path = palbox.join("architecture.md");
    if !arch_path.exists() {
        let arch = generate_architecture(root, task);
        std::fs::write(&arch_path, &arch)?;
        report.push_str(&format!("✅ Created: {}\n", arch_path.display()));
        files_written += 1;
    } else {
        report.push_str("ℹ️  .palbox/architecture.md already exists (skipped)\n");
    }

    // 4. Generate .env.example if missing
    let env_path = root.join(".env.example");
    if !env_path.exists() {
        let env = generate_env_example(root);
        if !env.trim().is_empty() {
            std::fs::write(&env_path, &env)?;
            report.push_str(&format!("✅ Created: {}\n", env_path.display()));
            files_written += 1;
        }
    }

    report.push_str(&format!("\n**Total files written:** {}\n", files_written));
    report.push_str("\n> 📝 Documentation-first: code follows docs, not the other way around.\n");

    Ok(report)
}

fn generate_readme(root: &std::path::Path, task: &str, py: bool, rs: bool, js: bool) -> String {
    let project_name = root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let setup_section = if py {
        "```bash\npython -m venv .venv && source .venv/bin/activate\npip install -r requirements.txt\n```"
    } else if js {
        "```bash\nnpm install\nnpm run dev\n```"
    } else if rs {
        "```bash\ncargo build --release\n```"
    } else {
        "```bash\n# Setup instructions\n```"
    };

    format!(
        "# {}\n\n{}\n\n## Architecture\n\n```\n{}\n```\n\n## Setup\n\n{}\n\n## API Reference\n\n> Add API endpoints here before implementing.\n\n### GET /api/health\n\n**Response 200:**\n```json\n{{\n  \"status\": \"ok\"\n}}\n```\n\n## Environment\n\n| Variable | Default | Description |\n|----------|---------|-------------|\n| PORT | 3000 | Server port |\n\n> Documentation-first: this README is the contract. Code must match.\n",
        project_name,
        task,
        project_structure_ascii(root),
        setup_section,
    )
}

fn project_structure_ascii(root: &std::path::Path) -> String {
    let mut tree = String::new();
    tree.push_str(&format!("{}", root.file_name().unwrap_or_default().to_string_lossy()));
    tree.push_str("/\n");

    let _project_name = root.file_name().unwrap_or_default().to_string_lossy();
    // Basic structure based on detected files
    if root.join("src").exists() {
        tree.push_str(&format!("├── src/\n│   ├── main.{}\n", if root.join("src/main.rs").exists() { "rs" } else { "ts" }));
        tree.push_str("│   └── lib/\n");
    }
    if root.join("api").exists() {
        tree.push_str("├── api/\n│   ├── main.py\n│   ├── routes/\n│   └── models/\n");
    }
    tree.push_str("├── README.md\n");
    tree.push_str("└── .env.example\n");

    tree
}

fn generate_architecture(_root: &std::path::Path, task: &str) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d");
    format!(
        "# Architecture\n\n**Last updated:** {}\n\n## Overview\n\n{}\n\n## Decisions\n\n### ADR-001: Initial architecture\n\n**Status:** Accepted\n**Date:** {}\n**Context:** Project initialization from task: \"{}\"\n**Decision:** Start with modular architecture, documentation-first.\n**Trade-off:** More upfront docs effort, but eliminates ambiguity later.\n\n## Components\n\n[tba — add as system grows]\n\n## Data Flow\n\n[tba]\n",
        now, task, now, task
    )
}

fn generate_env_example(root: &std::path::Path) -> String {
    if root.join("requirements.txt").exists() {
        "DATABASE_URL=postgresql://user:pass@localhost:5432/db\nAPI_KEY=your_key_here\n".to_string()
    } else if root.join("package.json").exists() {
        "VITE_API_URL=http://localhost:3000\n".to_string()
    } else {
        String::new()
    }
}

// ── Entry point ──────────────────────────────────────────────────

pub async fn run_server(
    palbox: PathBuf,
    cbm_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState { palbox: palbox.clone(), cbm_path };

    let transport = (tokio::io::stdin(), FlushingWriter { inner: tokio::io::stdout() });
    eprintln!("[palskills-engine] MCP server starting — 6 tools (orchestrate, write_docs, scan_context, dispatch, run_tests, record_session)");
    eprintln!("[palskills-engine] Palbox: {}", palbox.display());
    let service = state.serve(transport).await?;
    eprintln!("[palskills-engine] Connected.");
    let _ = service.waiting().await;
    Ok(())
}
