//! Planner (Jetdragon) — generate implementation plans from tasks.

use std::path::Path;
use anyhow::Result;

/// Generate a development plan for a task.
/// If yes=false, returns clarifying questions first.
pub fn generate_plan(project_root: &Path, task: &str, yes: bool) -> Result<String> {
    let date = chrono::Local::now().format("%Y-%m-%d");
    let palbox_exists = project_root.join(".palbox").exists();

    let mut plan = format!(
        "# Plan: {}\n**Date:** {}\n**Status:** {}\n\n",
        task, date,
        if yes { "APPROVED" } else { "DRAFT — awaiting feedback" }
    );

    // Add context section if palbox exists
    if palbox_exists {
        plan.push_str("## Knowledge Graph Context\n");
        plan.push_str("- [[architecture]] — Project structure\n");
        plan.push_str("- [[methods]] — Coding conventions\n");
        plan.push_str("\n");
    }

    // Overview
    plan.push_str("## Overview\n");
    plan.push_str(&format!("[Implementation plan for: {}]\n\n", task));

    // Scope
    plan.push_str("## Scope\n");
    plan.push_str("- **In scope:** [to be determined]\n");
    plan.push_str("- **Out of scope:** [to be determined]\n\n");

    // Tasks
    plan.push_str("## Tasks (ordered)\n");
    plan.push_str("### Task 1: Setup & Context\n");
    plan.push_str("- **What:** Understand existing code, prepare workspace\n");
    plan.push_str("- **Verification:** Tests pass before changes\n\n");
    plan.push_str("### Task 2: Core Implementation\n");
    plan.push_str(&format!("- **What:** Implement {}\n", task));
    plan.push_str("- **Verification:** New functionality works\n\n");
    plan.push_str("### Task 3: Tests & Cleanup\n");
    plan.push_str("- **What:** Write tests, refactor, documentation\n");
    plan.push_str("- **Verification:** All tests pass, lint clean\n\n");

    // If not auto-approved, add questions
    if !yes {
        plan.push_str("## Open Questions\n");
        plan.push_str("1. Scope: are there edge cases to handle?\n");
        plan.push_str("2. Integration: does this touch existing modules?\n");
        plan.push_str("3. Priority: is this P0 or can parts be deferred?\n\n");
        plan.push_str("---\n");
        plan.push_str("_Reply with answers or say **Gas** to proceed with defaults._\n");
    } else {
        plan.push_str("## Codex Prompt\n");
        plan.push_str(&format!("Implement: {}\n", task));
        plan.push_str("- Follow SOLID + SRP\n");
        plan.push_str("- Write tests before implementation (TDD)\n");
        plan.push_str("- Keep all code in English\n");
        plan.push_str("- Document public APIs\n");
    }

    Ok(plan)
}
