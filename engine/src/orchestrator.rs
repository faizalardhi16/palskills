//! Orchestrator (Astralym) — determine development flow from task intent.
//!
//! Uses CBM FIRST for code discovery, fallback to grep only if CBM unavailable.

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::cbm_bridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    pub flow: Vec<String>,
    pub confidence: u8,
    pub question: Option<String>,
    pub summary: String,
    pub context_source: String,       // "cbm" | "grep" | "none"
    pub found_symbols: usize,
    pub found_files: usize,
}

impl Default for OrchestrationPlan {
    fn default() -> Self {
        OrchestrationPlan {
            flow: vec!["scan_context".to_string()],
            confidence: 70,
            question: None,
            summary: "Starting context scan...".to_string(),
            context_source: "none".to_string(),
            found_symbols: 0,
            found_files: 0,
        }
    }
}

/// Analyze a task prompt: detect intent, query CBM/grep, determine flow.
pub fn analyze(project_root: &Path, task: &str) -> anyhow::Result<OrchestrationPlan> {
    let task_lower = task.to_lowercase();
    let mut flow = vec!["scan_context".to_string()];
    let mut confidence: u8 = 85;

    // Detect intent
    let is_build = task_lower.contains("build") || task_lower.contains("create") || task_lower.contains("implement");
    let is_fix = task_lower.contains("fix") || task_lower.contains("bug") || task_lower.contains("debug");
    let is_review = task_lower.contains("review") || task_lower.contains("audit");
    let is_plan = task_lower.contains("plan") || task_lower.contains("design") || task_lower.contains("brainstorm");
    let is_test = task_lower.contains("test") || task_lower.contains("tdd");
    let is_docs = task_lower.contains("doc") || task_lower.contains("readme") || task_lower.contains("spec");

    let needs_db = task_lower.contains("database") || task_lower.contains("table") || task_lower.contains("schema")
        || task_lower.contains("migration") || task_lower.contains("model");
    let needs_ui = task_lower.contains("ui") || task_lower.contains("component") || task_lower.contains("page")
        || task_lower.contains("frontend") || task_lower.contains("screen") || task_lower.contains("button");
    let needs_api = task_lower.contains("api") || task_lower.contains("endpoint") || task_lower.contains("controller")
        || task_lower.contains("service") || task_lower.contains("backend") || task_lower.contains("route");
    let needs_auth = task_lower.contains("auth") || task_lower.contains("login") || task_lower.contains("jwt");

    // Palbox check
    let palbox_path = project_root.join(".palbox");
    let has_palbox = palbox_path.exists();
    let mut question = None;

    if !has_palbox {
        confidence -= 15;
        question = Some("No .palbox/ detected. Should I bootstrap the project first?".to_string());
        flow.push("init_project".to_string());
    }

    // ── CBM FIRST: query codebase for relevant symbols ──
    let keywords: Vec<&str> = task
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '/' || c == '.')
        .filter(|w| w.len() > 2)
        .collect();

    let (cbm_symbols, cbm_files, source) = cbm_bridge::smart_search(project_root, &keywords);

    // Adjust confidence based on CBM results
    match source.as_str() {
        "cbm" => {
            confidence += 10;  // CBM found code — high confidence
            if cbm_symbols.len() > 5 { confidence += 5; }
            log::info!("📦 CBM hit: {} symbols, {} files", cbm_symbols.len(), cbm_files.len());
        }
        "grep" => {
            confidence -= 10;  // Fallback to grep — lower confidence
            log::warn!("⚠  CBM miss — using grep. Index with 'paldeck index' for better results.");
        }
        _ => {
            confidence -= 15;  // No tool found anything
            log::warn!("⚠  No codebase context available.");
        }
    }

    // Build flow
    if is_docs {
        flow.push("write_docs".to_string());
        flow.push("record_session".to_string());
    } else if is_plan {
        flow.push("write_docs".to_string());
        flow.push("generate_plan".to_string());
        if needs_db { flow.push("design_schema".to_string()); }
        if needs_api { flow.push("architect_backend".to_string()); }
        if needs_ui { flow.push("componentize_ui".to_string()); }
    } else if is_build {
        flow.push("write_docs".to_string());
        if needs_db { flow.push("design_schema".to_string()); }
        if needs_api { flow.push("architect_backend".to_string()); }
        if needs_ui { flow.push("componentize_ui".to_string()); }
        flow.push("generate_plan".to_string());
        flow.push("dispatch_build".to_string());
        flow.push("record_session".to_string());
    } else if is_fix {
        flow.push("scan_context".to_string());
        flow.push("dispatch_build".to_string());
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else if is_review {
        flow.push("generate_plan".to_string());
    } else if is_test {
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else {
        question = Some(format!(
            "Not sure about intent: '{}'. Is this a build, fix, review, plan, or test task?",
            task
        ));
        confidence = 50;
    }

    if keywords.len() < 3 { confidence -= 5; }
    if needs_auth && needs_db && needs_api { confidence = 90; }

    // Cap confidence
    confidence = confidence.min(95);

    let context_label = if cbm_symbols.is_empty() && cbm_files.is_empty() {
        "none".to_string()
    } else {
        format!("{} symbols, {} files", cbm_symbols.len(), cbm_files.len())
    };

    let summary = format!(
        "Flow: {} | Palbox: {} | Context: {} ({}) | Confidence: {}%",
        flow.join(" → "),
        if has_palbox { "✓" } else { "✗" },
        source.to_uppercase(),
        context_label,
        confidence,
    );

    Ok(OrchestrationPlan {
        flow,
        confidence,
        question,
        summary,
        context_source: source,
        found_symbols: cbm_symbols.len(),
        found_files: cbm_files.len(),
    })
}
