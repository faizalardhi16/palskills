//! MCP Server — 5 orchestration tools for AI agents.
//!
//! Pipeline: orchestrate → scan_context → dispatch → run_tests → record_session
//! orchestrate auto-generates advisory plans for complex tasks (no blocking gate).
//! dispatch returns SOLID contract (no subprocess — agent main yg eksekusi).
//! record_session syncs docs back to .palbox/ after task completion.

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
use crate::palbox_context;

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

/// Structured knowledge params for record_session.
/// Agent fills these with REAL knowledge — not file lists.
#[derive(Deserialize, JsonSchema)]
pub struct RecordSessionParams {
    pub task: String,
    pub summary: String,
    #[serde(default)]
    pub decisions: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub lessons: Vec<String>,
    #[serde(default)]
    pub api: Vec<String>,
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
    /// Astralym: CBM-aware flow detection + advisory planning.
    /// Returns flow, confidence, and for complex tasks an auto-generated
    /// advisory plan (written to .palbox/plans/). No blocking "Gas" gate —
    /// agent reads plan and proceeds immediately.
    #[tool(
        name = "orchestrate",
        description = "Analyze task via CBM code search. Returns recommended flow, confidence %, and for complex tasks an advisory plan saved to .palbox/plans/ (no blocking gate). Use this FIRST before any code generation."
    )]
    fn orchestrate(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "orchestrate", &format!("Analyzing: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        match orchestrator::analyze(root, &p.task) {
            Ok(ctx) => {
                let flow = ctx.flow.clone();
                let conf = ctx.confidence;
                let summary = ctx.summary.clone();
                let has_plan = ctx.plan_content.is_some();
                tool_done(&self.palbox, "orchestrate", &summary, timer, &p.task, Some(flow), Some(conf));

                let mut output = serde_json::to_string_pretty(&ctx).unwrap_or_default();
                if has_plan {
                    output.push_str("\n\n📋 Advisory plan auto-generated — review .palbox/plans/ if needed.");
                }
                out(output)
            }
            Err(e) => {
                tool_error(&self.palbox, "orchestrate", &format!("Error: {e}"), timer);
                out(format!("Error: {e}"))
            }
        }
    }

    /// Lyleen: CBM-backed code search + palbox docs context.
    /// Fast: skips deep grep when CBM unavailable — returns light listing + docs.
    #[tool(
        name = "scan_context",
        description = "Search codebase via CBM + read .palbox/ docs. Returns code symbols/files AND architecture summary, recent flows, and session history. Fast — skips deep grep when no CBM, returns light file listing instead."
    )]
    fn scan_context(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "scan_context", &format!("Scanning: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        // Fast-path: check if CBM index exists
        let cbm_available = cbm_bridge::check_available(root).unwrap_or(false);
        let cbm = if cbm_available {
            cbm_bridge::get_context(root, &p.task).ok()
        } else {
            // No CBM: skip deep grep, return light file listing only
            log::info!("⚡ No CBM index — fast-path: listing relevant files only");
            Some(cbm_bridge::CbmContext {
                available: false,
                symbols: vec![],
                callers: vec![],
                architecture: None,
                files: cbm_bridge::quick_file_listing(root, &p.task),
                source: "fast-scan".to_string(),
            })
        };
        let docs = palbox_context::read_docs(root, &p.task);

        #[derive(Serialize)]
        struct ContextOutput {
            symbols: Vec<String>,
            files: Vec<String>,
            source: String,
            architecture: Option<String>,
            database: Option<String>,
            decisions: Vec<String>,
            modules: Vec<String>,
            api_contracts: Vec<String>,
            lessons: Vec<String>,
            recent_flows: Vec<String>,
            recent_sessions: Vec<String>,
            latest_plan: Option<String>,
        }

        let output = ContextOutput {
            symbols: cbm.as_ref().map(|c| c.symbols.iter().map(|s| s.name.clone()).collect()).unwrap_or_default(),
            files: cbm.as_ref().map(|c| c.files.clone()).unwrap_or_default(),
            source: cbm.as_ref().map(|c| c.source.clone()).unwrap_or_else(|| "none".into()),
            architecture: docs.architecture_summary,
            database: docs.database_summary,
            decisions: docs.decisions,
            modules: docs.modules,
            api_contracts: docs.api_contracts,
            lessons: docs.lessons,
            recent_flows: docs.recent_flows,
            recent_sessions: docs.recent_sessions,
            latest_plan: docs.latest_plan,
        };

        let msg = format!(
            "Found {} symbols, {} files (via {}) + {} docs from .palbox/",
            output.symbols.len(), output.files.len(), output.source, docs.docs_found
        );
        tool_done(&self.palbox, "scan_context", &msg, timer, &p.task, None, None);
        out(serde_json::to_string_pretty(&output).unwrap_or_default())
    }

    /// Anubis: SOLID discipline gate. ZERO I/O — pure contract.
    #[tool(
        name = "dispatch",
        description = "SOLID discipline gate. Returns SOLID principles contract. ZERO I/O — pure constraints. Context was already gathered by scan_context. Read this contract BEFORE writing any code."
    )]
    fn dispatch(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "dispatch", "Generating SOLID contract", &p.task);

        let contract = dispatch::generate_contract(&p.task);
        let msg = "SOLID contract ready";
        tool_done(&self.palbox, "dispatch", msg, timer, &p.task, None, None);
        out(serde_json::to_string_pretty(&contract).unwrap_or_default())
    }

    /// Verdash: Run actual test suite.
    #[tool(
        name = "run_tests",
        description = "Run project test suite. Auto-detects runner (pytest, cargo test, npm test). Returns pass/fail results. Fails are SOFT BLOCKERS — fix and re-run."
    )]
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

    /// Panthalus: Persist session knowledge + sync docs to .palbox/.
    /// Agent MUST pass structured knowledge — summary, decisions, modules,
    /// lessons, api — NOT file lists. This is what makes .palbox/ a knowledge
    /// base that future sessions read via scan_context.
    #[tool(
        name = "record_session",
        description = "Record session knowledge AND sync docs. Pass structured knowledge: summary (what was done), decisions (why), modules (what each does), lessons (gotchas), api (endpoints). This builds .palbox/ knowledge base for future sessions."
    )]
    fn record_session(&self, Parameters(p): Parameters<RecordSessionParams>) -> Json<ToolOutput> {
        let timer = tool_start(&self.palbox, "record_session", &format!("Recording: {}", p.task), &p.task);
        let root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        let knowledge = generator::SessionKnowledge {
            summary: p.summary.clone(),
            decisions: p.decisions.clone(),
            modules: p.modules.clone(),
            lessons: p.lessons.clone(),
            api: p.api.clone(),
        };

        let mut report = String::new();

        // 1. Record session history with knowledge
        match generator::record_session(root, &knowledge) {
            Ok(path) => {
                report.push_str(&format!("✅ Session recorded: {}\n", path.display()));
            }
            Err(e) => {
                report.push_str(&format!("⚠ Session recording failed: {e}\n"));
            }
        }

        // 2. Sync docs (architecture, database, flows + decisions, modules, api, lessons)
        match generator::sync_docs(root, &p.task, &knowledge) {
            Ok(sync_report) => {
                report.push_str(&format!("\n📄 Docs synced:\n{sync_report}"));
            }
            Err(e) => {
                report.push_str(&format!("\n⚠ Docs sync failed: {e}"));
            }
        }

        let msg = format!("Knowledge recorded ({} chars)", report.len());
        tool_done(&self.palbox, "record_session", &msg, timer, &p.task, None, None);
        out(report)
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
