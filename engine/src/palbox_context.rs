//! Palbox Context — reads .palbox/ documentation to enrich scan_context.
//!
//! After record_session syncs docs (architecture.md, database.md, flows/,
//! history/), this module reads them back so the next session has full context.
//! CBM covers code; palbox_context covers decisions, architecture, and history.

use std::path::Path;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PalboxContext {
    pub architecture_summary: Option<String>,
    pub database_summary: Option<String>,
    pub decisions: Vec<String>,
    pub modules: Vec<String>,
    pub api_contracts: Vec<String>,
    pub lessons: Vec<String>,
    pub recent_flows: Vec<String>,
    pub recent_sessions: Vec<String>,
    pub latest_plan: Option<String>,
    pub docs_found: usize,
}

/// Read relevant .palbox/ documentation for context injection.
pub fn read_docs(project_root: &Path, _task: &str) -> PalboxContext {
    let palbox = project_root.join(".palbox");
    let mut ctx = PalboxContext {
        architecture_summary: None,
        database_summary: None,
        decisions: Vec::new(),
        modules: Vec::new(),
        api_contracts: Vec::new(),
        lessons: Vec::new(),
        recent_flows: Vec::new(),
        recent_sessions: Vec::new(),
        latest_plan: None,
        docs_found: 0,
    };

    if !palbox.exists() {
        return ctx;
    }

    // 1. Architecture.md — take last 2 update sections
    let arch_path = palbox.join("architecture.md");
    if let Ok(content) = std::fs::read_to_string(&arch_path) {
        let summary = extract_last_updates(&content, 2);
        if !summary.is_empty() {
            ctx.architecture_summary = Some(summary);
            ctx.docs_found += 1;
        }
    }

    // 2. Database.md — take last update section
    let db_path = palbox.join("database.md");
    if let Ok(content) = std::fs::read_to_string(&db_path) {
        let summary = extract_last_updates(&content, 1);
        if !summary.is_empty() {
            ctx.database_summary = Some(summary);
            ctx.docs_found += 1;
        }
    }

    // 3. Decisions.md — last 5 ADRs (agent-written knowledge)
    let decisions_path = palbox.join("decisions.md");
    if let Ok(content) = std::fs::read_to_string(&decisions_path) {
        ctx.decisions = content
            .lines()
            .filter(|l| l.trim().starts_with("- "))
            .map(|l| l.trim().trim_start_matches("- ").to_string())
            .take(5)
            .collect();
        if !ctx.decisions.is_empty() {
            ctx.docs_found += 1;
        }
    }

    // 4. Modules/ — list per-module knowledge
    let modules_dir = palbox.join("modules");
    if modules_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&modules_dir) {
            let mut modules: Vec<String> = entries
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let path = e.path();
                    std::fs::read_to_string(&path).ok().map(|content| {
                        let purpose = content
                            .lines()
                            .skip_while(|l| !l.trim().starts_with("## Purpose"))
                            .nth(1)
                            .map(|l| l.trim().to_string())
                            .unwrap_or_default();
                        format!("- **{}**: {}", name.trim_end_matches(".md"), purpose)
                    })
                })
                .collect();
            modules.sort();
            ctx.modules = modules;
            if !ctx.modules.is_empty() {
                ctx.docs_found += 1;
            }
        }
    }

    // 5. API.md — last 10 endpoint contracts
    let api_path = palbox.join("api.md");
    if let Ok(content) = std::fs::read_to_string(&api_path) {
        ctx.api_contracts = content
            .lines()
            .filter(|l| l.trim().starts_with("- `"))
            .map(|l| l.trim().trim_start_matches("- ").to_string())
            .take(10)
            .collect();
        if !ctx.api_contracts.is_empty() {
            ctx.docs_found += 1;
        }
    }

    // 6. Lessons.md — last 5 lessons
    let lessons_path = palbox.join("lessons.md");
    if let Ok(content) = std::fs::read_to_string(&lessons_path) {
        ctx.lessons = content
            .lines()
            .filter(|l| l.trim().starts_with("- "))
            .map(|l| l.trim().trim_start_matches("- ").to_string())
            .take(5)
            .collect();
        if !ctx.lessons.is_empty() {
            ctx.docs_found += 1;
        }
    }

    // 7. Flows/ — list recent flow files (last 3)
    let flows_dir = palbox.join("flows");
    if flows_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&flows_dir) {
            let mut files: Vec<_> = entries
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let path = e.path();
                    std::fs::read_to_string(&path).ok().map(|content| {
                        format!("- **{}**: {}", name.trim_end_matches(".md"), first_line(&content))
                    })
                })
                .collect();
            files.sort_by(|a, b| b.cmp(a)); // newest-ish first
            ctx.recent_flows = files.into_iter().take(3).collect();
            if !ctx.recent_flows.is_empty() {
                ctx.docs_found += 1;
            }
        }
    }

    // 8. History/ — last 3 session summaries
    let history_dir = palbox.join("history");
    if history_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&history_dir) {
            let mut files: Vec<_> = entries
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let path = e.path();
                    std::fs::read_to_string(&path).ok().map(|content| {
                        let task = extract_field(&content, "## Summary");
                        format!("- **{}**: {}", name.trim_end_matches(".md"), task)
                    })
                })
                .collect();
            files.sort_by(|a, b| b.cmp(a)); // newest first
            ctx.recent_sessions = files.into_iter().take(3).collect();
            if !ctx.recent_sessions.is_empty() {
                ctx.docs_found += 1;
            }
        }
    }

    // 9. Plans/ — most recent advisory plan (first 20 lines)
    let plans_dir = palbox.join("plans");
    if plans_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plans_dir) {
            let mut files: Vec<_> = entries
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .collect();
            files.sort_by_key(|e| e.file_name());
            if let Some(latest) = files.last() {
                if let Ok(content) = std::fs::read_to_string(latest.path()) {
                    let preview: String = content.lines().take(20).collect::<Vec<_>>().join("\n");
                    if !preview.is_empty() {
                        ctx.latest_plan = Some(preview);
                        ctx.docs_found += 1;
                    }
                }
            }
        }
    }

    ctx
}

/// Extract the last N "## Update:" sections from a doc.
fn extract_last_updates(content: &str, count: usize) -> String {
    let sections: Vec<&str> = content.split("## Update:").collect();
    if sections.len() <= 1 {
        // No update sections, return first 300 chars as summary
        return content.lines().take(10).collect::<Vec<_>>().join("\n");
    }
    sections
        .iter()
        .rev()
        .take(count)
        .rev()
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

/// Get first non-empty line of content.
fn first_line(content: &str) -> String {
    content
        .lines()
        .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .unwrap_or("(empty)")
        .trim()
        .to_string()
}

/// Extract a field value from markdown (e.g., "## Task" → next line).
fn extract_field(content: &str, field: &str) -> String {
    let mut found = false;
    for line in content.lines() {
        if found {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if line.trim() == field {
            found = true;
        }
    }
    "(no task)".to_string()
}
