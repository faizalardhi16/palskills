//! Orchestrator (Astralym) — determine development flow from task intent.
//!
//! Uses CBM FIRST for code discovery, fallback to grep only if CBM unavailable.
//! For complex tasks (confidence < 70% or build/plan intent), auto-generates
//! an advisory plan saved to .palbox/plans/ — no blocking "Gas" gate.

use std::path::{Path, PathBuf};
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
    /// Advisory plan — generated for complex tasks, None for simple ones.
    /// Agent reads this but is NOT blocked waiting for "Gas".
    pub plan_content: Option<String>,
    pub plan_path: Option<String>,
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
            plan_content: None,
            plan_path: None,
        }
    }
}

/// Analyze a task prompt: detect intent, query CBM/grep, determine flow.
/// Auto-generates advisory plan for complex tasks (no blocking gate).
pub fn analyze(project_root: &Path, task: &str, palbox_active: bool) -> anyhow::Result<OrchestrationPlan> {
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
    let is_complex = task_lower.split_whitespace().count() > 6
        || needs_db && needs_api
        || needs_ui && needs_api
        || is_build && (needs_db || needs_ui || needs_api);

    // Palbox check
    let palbox_path = project_root.join(".palbox");
    let has_palbox = palbox_path.exists();
    let mut question = None;

    if !has_palbox {
        confidence -= 15;
        question = Some("No .palbox/ detected — PASSIVE mode (no recording, no knowledge context). Run 'palskills-engine init' to enable the knowledge base.".to_string());
        if !palbox_active {
            // Passive mode: never suggest init_project flow or write plans
            // (flow stays scan_context → dispatch only)
        } else {
            flow.push("init_project".to_string());
        }
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

    // Build flow — streamlined: 5-tool pipeline, no plan gate
    if is_docs {
        flow.push("dispatch".to_string());
        flow.push("record_session".to_string());
    } else if is_plan {
        flow.push("dispatch".to_string());
        flow.push("record_session".to_string());
    } else if is_build {
        flow.push("dispatch".to_string());
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else if is_fix {
        flow.push("dispatch".to_string());
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else if is_review {
        flow.push("dispatch".to_string());
    } else if is_test {
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else {
        question = Some(format!(
            "Not sure about intent: '{}'. Is this a build, fix, review, or test task?",
            task
        ));
        confidence = 50;
    }

    if keywords.len() < 3 { confidence -= 5; }
    if needs_auth && needs_db && needs_api { confidence = 90; }

    // Cap confidence
    confidence = confidence.min(95);

    // ── Advisory plan for complex / low-confidence tasks ──
    // Passive mode (no .palbox): NEVER write plan files — would create folders.
    let (plan_content, plan_path) = if palbox_active && (confidence <= 70 || is_complex || is_build || is_plan) {
        let (content, path) = generate_advisory_plan(
            project_root,
            task,
            &keywords,
            cbm_symbols.len(),
            cbm_files.len(),
            &flow,
            needs_db,
            needs_ui,
            needs_api,
        );
        log::info!("📋 Advisory plan written: {}", path.display());
        (Some(content), Some(path.display().to_string()))
    } else {
        (None, None)
    };

    let context_label = if cbm_symbols.is_empty() && cbm_files.is_empty() {
        "none".to_string()
    } else {
        format!("{} symbols, {} files", cbm_symbols.len(), cbm_files.len())
    };

    let summary = format!(
        "Flow: {} | Palbox: {} | Context: {} ({}) | Confidence: {}%{}",
        flow.join(" → "),
        if has_palbox { "✓" } else { "✗" },
        source.to_uppercase(),
        context_label,
        confidence,
        if plan_content.is_some() { " | 📋 advisory plan generated" } else { "" }
    );

    Ok(OrchestrationPlan {
        flow,
        confidence,
        question,
        summary,
        context_source: source,
        found_symbols: cbm_symbols.len(),
        found_files: cbm_files.len(),
        plan_content,
        plan_path,
    })
}

/// Generate advisory execution plan, write to .palbox/plans/.
/// This is NOT a blocking gate — agent reads it inline and proceeds.
fn generate_advisory_plan(
    root: &Path,
    task: &str,
    keywords: &[&str],
    symbol_count: usize,
    file_count: usize,
    flow: &[String],
    needs_db: bool,
    needs_ui: bool,
    needs_api: bool,
) -> (String, PathBuf) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let plan_name: String = task
        .chars()
        .take(50)
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();

    let mut sections = String::new();

    // Risk + approach
    sections.push_str(&format!(
        "**Complexity:** {} keywords, {} symbols, {} files\n\n",
        keywords.len(), symbol_count, file_count
    ));

    sections.push_str("## Files to touch\n\n[tba]\n\n");

    sections.push_str("## Approach\n\n1. Read existing code (see context above)\n");
    let mut step = 2;
    if needs_db { sections.push_str(&format!("{}. Design/update schema\n", step)); step += 1; }
    if needs_api { sections.push_str(&format!("{}. Implement API layer\n", step)); step += 1; }
    if needs_ui { sections.push_str(&format!("{}. Build UI components\n", step)); step += 1; }
    sections.push_str(&format!("{}. Write tests\n{}. Verify + record\n\n", step, step + 1));

    sections.push_str("## Risk assessment\n\n");
    sections.push_str(&format!("- **Confidence:** see orchestrate output\n"));
    sections.push_str("- Files to touch are [tba] — review before modifying\n\n");

    let plan_content = format!(
        "# Advisory Plan: {}\n\n**Date:** {}\n**Pipeline:** {}\n\n{}---\n> 📋 Auto-generated advisory plan. Agent proceeds immediately.\n> Review .palbox/plans/ for details — no blocking \"Gas\" gate.\n",
        task,
        now,
        flow.join(" → "),
        sections
    );

    // Write to .palbox/plans/
    let dir = root.join(".palbox").join("plans");
    let _ = std::fs::create_dir_all(&dir);
    let filename = format!("{}-{}.md", chrono::Local::now().format("%Y-%m-%d"), plan_name.trim_matches('-'));
    let path = dir.join(&filename);
    let _ = std::fs::write(&path, &plan_content);

    (plan_content, path)
}
