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
    pub recent_flows: Vec<String>,
    pub recent_sessions: Vec<String>,
    pub docs_found: usize,
}

/// Read relevant .palbox/ documentation for context injection.
pub fn read_docs(project_root: &Path, _task: &str) -> PalboxContext {
    let palbox = project_root.join(".palbox");
    let mut ctx = PalboxContext {
        architecture_summary: None,
        database_summary: None,
        recent_flows: Vec::new(),
        recent_sessions: Vec::new(),
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

    // 3. Flows/ — list recent flow files (last 3)
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

    // 4. History/ — last 3 session summaries
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
                        let task = extract_field(&content, "## Task");
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
