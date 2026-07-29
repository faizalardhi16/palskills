//! Orchestrator (Astralym) — determine development flow from task intent.

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::cbm_bridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    /// Ordered list of skills to invoke
    pub flow: Vec<String>,
    /// Confidence percentage
    pub confidence: u8,
    /// Clarifying question, if scope is ambiguous
    pub question: Option<String>,
    /// Summary of what was detected
    pub summary: String,
}

impl Default for OrchestrationPlan {
    fn default() -> Self {
        OrchestrationPlan {
            flow: vec!["scan_context".to_string()],
            confidence: 70,
            question: None,
            summary: "Starting context scan...".to_string(),
        }
    }
}

/// Analyze a task prompt and determine which skills are needed.
pub fn analyze(project_root: &Path, task: &str) -> anyhow::Result<OrchestrationPlan> {
    let task_lower = task.to_lowercase();
    let mut flow = vec!["scan_context".to_string()];
    let mut confidence: u8 = 85;
    let mut question = None;

    // Determine intent from keywords
    let is_build = task_lower.contains("build") || task_lower.contains("create") || task_lower.contains("implement");
    let is_fix = task_lower.contains("fix") || task_lower.contains("bug") || task_lower.contains("debug");
    let is_review = task_lower.contains("review") || task_lower.contains("audit");
    let is_plan = task_lower.contains("plan") || task_lower.contains("design") || task_lower.contains("brainstorm");
    let is_test = task_lower.contains("test") || task_lower.contains("tdd");

    let needs_db = task_lower.contains("database") || task_lower.contains("table") || task_lower.contains("schema")
        || task_lower.contains("migration") || task_lower.contains("model");
    let needs_ui = task_lower.contains("ui") || task_lower.contains("component") || task_lower.contains("page")
        || task_lower.contains("frontend") || task_lower.contains("screen") || task_lower.contains("button");
    let needs_api = task_lower.contains("api") || task_lower.contains("endpoint") || task_lower.contains("controller")
        || task_lower.contains("service") || task_lower.contains("backend") || task_lower.contains("route");
    let needs_auth = task_lower.contains("auth") || task_lower.contains("login") || task_lower.contains("jwt");

    // Check palbox for existing context
    let palbox_path = project_root.join(".palbox");
    let has_palbox = palbox_path.exists();

    if !has_palbox {
        confidence -= 15;
        question = Some("No .palbox/ detected. Should I bootstrap the project first?".to_string());
        flow.push("init_project".to_string());
    }

    // Check CBM availability
    let has_cbm = cbm_bridge::check_available(project_root).unwrap_or(false);
    if !has_cbm {
        confidence -= 10;
    }

    // Build flow based on intent
    if is_plan {
        flow.push("generate_plan".to_string());
        if needs_db { flow.push("design_schema".to_string()); }
        if needs_api { flow.push("architect_backend".to_string()); }
        if needs_ui { flow.push("componentize_ui".to_string()); }
    } else if is_build {
        // Full pipeline
        if needs_db { flow.push("design_schema".to_string()); }
        if needs_api { flow.push("architect_backend".to_string()); }
        if needs_ui { flow.push("componentize_ui".to_string()); }
        flow.push("generate_plan".to_string());
        flow.push("dispatch_build".to_string());
        flow.push("record_session".to_string());
    } else if is_fix {
        flow.push("generate_plan".to_string());
        flow.push("dispatch_build".to_string());
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else if is_review {
        flow.push("generate_plan".to_string());
    } else if is_test {
        flow.push("run_tests".to_string());
        flow.push("record_session".to_string());
    } else {
        // Unknown intent → ask user
        question = Some(format!(
            "Not sure about intent: '{}'. Is this a build, fix, review, plan, or test task?",
            task
        ));
        confidence = 50;
    }

    // If scope is ambiguous, lower confidence
    let keywords: Vec<&str> = task.split_whitespace().collect();
    if keywords.len() < 3 {
        confidence -= 5;
    }

    if needs_auth && needs_db && needs_api {
        confidence = 90;
    }

    let summary = format!(
        "Flow: {} | Palbox: {} | CBM: {} | Confidence: {}%",
        flow.join(" → "),
        if has_palbox { "✓" } else { "✗" },
        if has_cbm { "✓" } else { "✗" },
        confidence,
    );

    Ok(OrchestrationPlan {
        flow,
        confidence,
        question,
        summary,
    })
}
