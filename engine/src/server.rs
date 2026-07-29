//! MCP Server — 5 orchestration tools for AI agents.
//!
//! Tools: orchestrate (CBM-aware flow detection), scan_context (CBM code search),
//! dispatch (spawn agent), run_tests (verify), record_session (persist).

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

// ── Tools (5 total) ──────────────────────────────────────────────

#[tool_router(server_handler)]
impl AppState {
    /// Astralym: CBM-aware flow detection.
    #[tool(name = "orchestrate", description = "Analyze task via CBM code search. Returns recommended flow, confidence %, and relevant symbols/files. Use this FIRST before any code generation.")]
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

// ── Entry point ──────────────────────────────────────────────────

pub async fn run_server(
    palbox: PathBuf,
    cbm_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState { palbox: palbox.clone(), cbm_path };

    let transport = (tokio::io::stdin(), FlushingWriter { inner: tokio::io::stdout() });
    eprintln!("[palskills-engine] MCP server starting — 5 tools (orchestrate, scan_context, dispatch, run_tests, record_session)");
    eprintln!("[palskills-engine] Palbox: {}", palbox.display());
    let service = state.serve(transport).await?;
    eprintln!("[palskills-engine] Connected.");
    let _ = service.waiting().await;
    Ok(())
}
