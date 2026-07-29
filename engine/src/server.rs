//! MCP Server — 11 palskills tools as MCP tools for AI agents.
//!
//! Tools: orchestrate, scan_context, generate_plan, generate_prd,
//! design_system, architect_backend, design_schema, componentize_ui,
//! dispatch_build, run_tests, record_session

use std::path::PathBuf;
use std::sync::Mutex;

use rmcp::{tool_router, tool, ServiceExt, handler::server::wrapper::{Parameters, Json}};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::palbox_graph;
use crate::cbm_bridge;
use crate::orchestrator;
use crate::planner;
use crate::generator;
use crate::dispatch;

/// Shared state for all MCP tools
pub struct AppState {
    pub palbox: PathBuf,
    pub cbm_path: PathBuf,
    pub palbox_conn: Mutex<rusqlite::Connection>,
}

// ── Tool parameter types ──────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct TaskParams {
    pub task: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ConfirmParams {
    pub task: String,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct DesignParams {
    pub task: String,
}

/// Cursor MCP requires outputSchema type: "object". All tools wrap string result.
#[derive(Serialize, JsonSchema)]
pub struct ToolOutput {
    pub result: String,
}

fn out(s: String) -> Json<ToolOutput> {
    Json(ToolOutput { result: s })
}

// ── Auto-flush stdout wrapper (fix Cursor buffering) ──────────────

use tokio::io::AsyncWrite;
use std::pin::Pin;
use std::task::{Context, Poll};

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

// ── Tools ─────────────────────────────────────────────────────────

#[tool_router(server_handler)]
impl AppState {
    /// Astralym: Analyze task, determine flow, return orchestration plan.
    #[tool(name = "orchestrate", description = "Astralym: analyze a development task and determine which skills are needed. Returns flow recommendation with confidence score and clarifying questions if needed.")]
    fn orchestrate(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let result = match orchestrator::analyze(project_root, &p.task) {
            Ok(ctx) => serde_json::to_string_pretty(&ctx).unwrap_or_default(),
            Err(e) => format!("Error: {}", e),
        };
        out(result)
    }

    /// Lyleen: Scan palbox knowledge graph + CBM for relevant context.
    #[tool(name = "scan_context", description = "Lyleen: scan the palbox knowledge graph and CBM codebase graph for context relevant to a task. Returns seeds, neighbors, and code symbols.")]
    fn scan_context(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));

        // Palbox graph
        let palbox = match self.palbox_conn.lock() {
            Ok(conn) => {
                match palbox_graph::scan_context(&conn, &p.task) {
                    Ok(ctx) => Some(ctx),
                    Err(e) => { log::warn!("Palbox scan error: {}", e); None }
                }
            }
            Err(e) => { log::warn!("Palbox lock error: {}", e); None }
        };

        // CBM graph
        let cbm = cbm_bridge::get_context(project_root, &p.task).ok();

        #[derive(Serialize)]
        struct ContextOutput {
            palbox: Option<palbox_graph::ContextResult>,
            cbm: Option<cbm_bridge::CbmContext>,
            total_nodes: usize,
            total_symbols: usize,
            total_files: usize,
            context_source: String,
        }

        let output = ContextOutput {
            total_nodes: palbox.as_ref().map(|p| p.seeds.len() + p.neighbors.len()).unwrap_or(0),
            total_symbols: cbm.as_ref().map(|c| c.symbols.len()).unwrap_or(0),
            total_files: cbm.as_ref().map(|c| c.files.len()).unwrap_or(0),
            context_source: cbm.as_ref().map(|c| c.source.clone()).unwrap_or_default(),
            palbox,
            cbm,
        };

        out(serde_json::to_string_pretty(&output).unwrap_or_default())
    }

    /// Jetdragon: Generate implementation plan from task description.
    #[tool(name = "generate_plan", description = "Jetdragon: brainstorm a development task and generate a detailed implementation plan. Asks clarifying questions if needed. Saves to .palbox/plans/.")]
    fn generate_plan(&self, Parameters(p): Parameters<ConfirmParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        match planner::generate_plan(project_root, &p.task, p.yes) {
            Ok(plan) => {
                let path = generator::save_plan(project_root, &p.task, &plan).unwrap_or_default();
                out(format!("Plan saved: {}\n\n{}", path.display(), plan))
            }
            Err(e) => out(format!("Error: {}", e)),
        }
    }

    /// Quivern: Generate PRD from discussion.
    #[tool(name = "generate_prd", description = "Quivern: generate a Product Requirement Document through collaborative discussion. Saves to .palbox/prds/.")]
    fn generate_prd(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let prd = format!("# PRD: {}\n**Generated:** {}\n\n## Problem Statement\n[tba]\n\n## User Stories\n|| # | As a... | I want to... | So that... | Priority |\n|---|---------|-------------|------------|----------|\n| US-1 | user | [action] | [goal] | P0 |\n\n## Requirements\n### Functional\n|| ID | Requirement | Priority |\n|----|-------------|----------|\n| FR-1 | [requirement] | P0 |\n\n### Non-Functional\n|| ID | Requirement | Target |\n|----|-------------|--------|\n| NFR-1 | Performance | <200ms |\n\n## Success Metrics\n[tba]\n\n## Scope\n- **MVP:** [tba]\n- **Out of scope:** [tba]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d")
        );
        let path = generator::save_prd(project_root, &p.task, &prd).unwrap_or_default();
        out(format!("PRD saved: {}\n\n{}", path.display(), prd))
    }

    /// Elphidran: Design system recommendations.
    #[tool(name = "design_system", description = "Elphidran: recommend design tokens, color system, typography, and component style patterns for the project.")]
    fn design_system(&self, Parameters(p): Parameters<DesignParams>) -> Json<ToolOutput> {
        out(format!("# Design System\n**Task:** {}\n**Generated:** {}\n\n## Color Palette\n|| Token | Value | Usage |\n|-------|-------|-------|\n| primary | #3B82F6 | CTAs, links |\n| background | #0F172A | Main BG (dark) |\n| surface | #1E293B | Cards, modals |\n\n## Typography\n- **Heading:** Inter, 24px Bold\n- **Body:** Inter, 16px Regular\n- **Code:** JetBrains Mono, 14px\n\n## Spacing\n- Base unit: 4px\n- Container padding: 16px\n- Section gap: 24px\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d")
        ))
    }

    /// Blazamut: Backend architecture specification.
    #[tool(name = "architect_backend", description = "Blazamut: design backend architecture — module decomposition, API contracts, class specifications, dependency injection tree. Saves to .palbox/architectures/.")]
    fn architect_backend(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let arch = format!("# Backend Architecture: {}\n**Date:** {}\n**Author:** Blazamut\n\n## Module Structure\n```\nsrc/\n├── controllers/   → HTTP concerns only\n├── services/      → Business logic only\n├── repositories/  → Data access only\n├── validators/    → Validation rules\n├── dto/          → Data shapes\n└── middleware/    → Cross-cutting\n```\n\n## API Contracts\n[tba — generated per endpoint]\n\n## Class Specifications\n[tba — SOLID + SRP per class]\n\n## Dependency Graph\n[tba — injection tree]\n\n## Error Hierarchy\n[tba — exception types]\n\n## Logging Strategy\n[tba — structured logging]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d")
        );
        let path = generator::save_architecture(project_root, &p.task, &arch).unwrap_or_default();
        out(format!("Architecture saved: {}\n\n{}", path.display(), arch))
    }

    /// Grizzbolt: Database schema design.
    #[tool(name = "design_schema", description = "Grizzbolt: design database schema — tables, columns, indexes, relationships, migrations. Saves to .palbox/schemas/.")]
    fn design_schema(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let schema = format!("# Database Schema: {}\n**Date:** {}\n**Author:** Grizzbolt\n\n## Tables\n[tba]\n\n## Indexes\n[tba]\n\n## Relationships\n[tba]\n\n## Migrations\n```sql\n-- UP\n-- [migration SQL]\n\n-- DOWN\n-- [rollback SQL]\n```\n\n## Performance Budget\n[tba]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d")
        );
        let path = generator::save_schema(project_root, &p.task, &schema).unwrap_or_default();
        out(format!("Schema saved: {}\n\n{}", path.display(), schema))
    }

    /// Astegon: Frontend component tree.
    #[tool(name = "componentize_ui", description = "Astegon: design frontend component hierarchy — page layout, component tree, props, state management. Saves to .palbox/components/.")]
    fn componentize_ui(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let comp = format!("# Component Tree: {}\n**Date:** {}\n**Author:** Astegon\n\n## Page Layout\n```\n[Page]\n├── [Header]\n│   └── [Navigation]\n├── [MainContent]\n│   ├── [Sidebar]\n│   └── [ContentArea]\n└── [Footer]\n```\n\n## Component Specs\n[tba — props, state, events per component]\n\n## State Management\n[tba — stores, context, reducers]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d")
        );
        let path = generator::save_component(project_root, &p.task, &comp).unwrap_or_default();
        out(format!("Component tree saved: {}\n\n{}", path.display(), comp))
    }

    /// Anubis: Dispatch to Codex CLI / agent for execution.
    #[tool(name = "dispatch_build", description = "Anubis: execute the implementation plan by dispatching to Codex CLI or similar AI agent. Passes enriched context and plan.")]
    fn dispatch_build(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        match dispatch::execute(project_root, &p.task, "plan approved", &Default::default()) {
            Ok(()) => out("✅ Dispatched. Agent is building...".to_string()),
            Err(e) => out(format!("❌ Dispatch failed: {}", e)),
        }
    }

    /// Verdash: TDD runner.
    #[tool(name = "run_tests", description = "Verdash: run tests in TDD mode (RED-GREEN-REFACTOR). Returns test results structured by pass/fail.")]
    fn run_tests(&self) -> Json<ToolOutput> {
        // Try common test commands
        let commands = ["npm test -- --reporter=min", "cargo test --quiet", "pytest -x --tb=short"];
        for cmd in &commands {
            if let Ok(result) = std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
            {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stdout.contains("pass") || stdout.contains("ok") || result.status.success() {
                    return out(format!("🧪 Tests:\n{}\n{}", stdout, stderr));
                }
            }
        }
        out("🧪 No test runner detected. Run tests manually.".to_string())
    }

    /// Panthalus: Record session to history.
    #[tool(name = "record_session", description = "Panthalus: record the current development session to .palbox/history/ for future context retrieval.")]
    fn record_session(&self, Parameters(p): Parameters<TaskParams>) -> Json<ToolOutput> {
        let project_root = self.palbox.parent().unwrap_or(std::path::Path::new("."));
        let session = format!("# Session: {}\n**Date:** {}\n**Author:** Panthalus\n\n## What was done\n[tba]\n\n## Files changed\n[tba]\n\n## Decisions made\n[tba]\n\n## Lessons learned\n[tba]\n",
            p.task,
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        );
        let path = generator::record_session(project_root, &p.task, &session).unwrap_or_default();
        // Re-index palbox graph
        if let Ok(_conn) = self.palbox_conn.lock() {
            let _ = palbox_graph::index(&self.palbox);
        }
        out(format!("✅ Session recorded: {}", path.display()))
    }
}

// ── Entry point for `serve` ──────────────────────────────────────

pub async fn run_server(
    palbox: PathBuf,
    cbm_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Index palbox graph on startup
    let _ = palbox_graph::index(&palbox);
    let conn = palbox_graph::open(&palbox)?;

    let state = AppState {
        palbox: palbox.clone(),
        cbm_path,
        palbox_conn: Mutex::new(conn),
    };

    let transport = (tokio::io::stdin(), FlushingWriter { inner: tokio::io::stdout() });
    eprintln!("[palskills] MCP server starting on stdio...");
    eprintln!("[palskills] Palbox: {}", palbox.display());
    let service = state.serve(transport).await?;
    eprintln!("[palskills] Connected. 11 tools registered.");
    let _ = service.waiting().await;
    Ok(())
}
