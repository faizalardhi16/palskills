//! Generator — persist pipeline state and session history.
//! State.md is the single source of truth for dashboard display.
//!
//! 6 skills: orchestrate, plan, scan_context, dispatch, run_tests, record_session

use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── State.md types ──────────────────────────────────────────────
// 6 skills: orchestrate → plan → scan_context → dispatch → run_tests → record_session

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillState {
    pub skill: String,
    pub status: String, // "idle" | "inprogress" | "done" | "error"
    pub started: Option<String>,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub updated: String,
    pub task: Option<String>,
    pub flow: Option<Vec<String>>,
    pub confidence: Option<u8>,
    pub skills: Vec<SkillState>,
    pub stats_nodes: Option<usize>,
    pub stats_symbols: Option<usize>,
    pub stats_files: Option<usize>,
}

const ALL_SKILLS: &[&str] = &[
    "orchestrate",
    "plan",
    "scan_context",
    "dispatch",
    "run_tests",
    "record_session",
];

/// Read current state from .palbox/State.md, or return empty default.
pub fn read_state(palbox: &Path) -> PipelineState {
    let path = palbox.join("State.md");
    if !path.exists() {
        return default_state();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    parse_state_md(&content)
}

fn default_state() -> PipelineState {
    PipelineState {
        updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        task: None,
        flow: None,
        confidence: None,
        skills: ALL_SKILLS
            .iter()
            .map(|s| SkillState {
                skill: s.to_string(),
                status: "idle".into(),
                started: None,
                duration_ms: None,
                message: None,
            })
            .collect(),
        stats_nodes: None,
        stats_symbols: None,
        stats_files: None,
    }
}

fn parse_state_md(content: &str) -> PipelineState {
    let mut state = default_state();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("**Task:**") {
            state.task = Some(line.replace("**Task:**", "").trim().to_string());
        }
        if line.starts_with("**Flow:**") {
            let flow_str = line.replace("**Flow:**", "").trim().to_string();
            state.flow = Some(flow_str.split(" → ").map(|s| s.trim().to_string()).collect());
        }
        if line.starts_with("**Confidence:**") {
            state.confidence = line.replace("**Confidence:**", "").trim().replace("%", "").parse().ok();
        }
        if line.starts_with("**Nodes:**") {
            state.stats_nodes = line.replace("**Nodes:**", "").trim().parse().ok();
        }
        if line.starts_with("**Symbols:**") {
            state.stats_symbols = line.replace("**Symbols:**", "").trim().parse().ok();
        }
        if line.starts_with("**Files:**") {
            state.stats_files = line.replace("**Files:**", "").trim().parse().ok();
        }
        for skill in ALL_SKILLS {
            if line.contains(skill) {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let status = parts[2].trim().to_lowercase();
                    let started = parts.get(3).filter(|s| s.trim() != "—").map(|s| s.trim().to_string());
                    let duration = parts.get(4).and_then(|s| s.trim().replace("ms", "").parse().ok());
                    if let Some(s) = state.skills.iter_mut().find(|s| s.skill == *skill) {
                        s.status = status;
                        s.started = started;
                        s.duration_ms = duration;
                    }
                }
            }
        }
    }
    state
}

/// Write current state to .palbox/State.md
pub fn write_state(palbox: &Path, state: &PipelineState) -> Result<()> {
    let path = palbox.join("State.md");
    let mut out = String::new();

    out.push_str("# Pipeline State\n\n");
    out.push_str(&format!("**Last updated:** {}\n", state.updated));
    if let Some(ref t) = state.task {
        out.push_str(&format!("**Task:** {}\n", t));
    }
    if let Some(ref f) = state.flow {
        out.push_str(&format!("**Flow:** {}\n", f.join(" → ")));
    }
    if let Some(c) = state.confidence {
        out.push_str(&format!("**Confidence:** {}%\n", c));
    }
    if let Some(n) = state.stats_nodes {
        out.push_str(&format!("**Nodes:** {}\n", n));
    }
    if let Some(s) = state.stats_symbols {
        out.push_str(&format!("**Symbols:** {}\n", s));
    }
    if let Some(f) = state.stats_files {
        out.push_str(&format!("**Files:** {}\n", f));
    }

    out.push_str("\n| Skill | Status | Started | Duration | Message |\n");
    out.push_str("|-------|--------|---------|----------|----------|\n");
    for s in &state.skills {
        let icon = match s.status.as_str() {
            "done" => "✅",
            "inprogress" => "🔄",
            "error" => "❌",
            _ => "⏳",
        };
        out.push_str(&format!(
            "| {} {} | {} | {} | {} | {} |\n",
            icon,
            s.skill,
            s.status,
            s.started.as_deref().unwrap_or("—"),
            s.duration_ms.map_or("—".to_string(), |d| format!("{}ms", d)),
            s.message.as_deref().unwrap_or(""),
        ));
    }

    std::fs::create_dir_all(palbox)?;
    std::fs::write(&path, out)?;
    Ok(())
}

/// Update a single skill's state and persist to State.md
pub fn update_skill_state(
    palbox: &Path,
    skill: &str,
    status: &str,
    message: Option<&str>,
    duration_ms: Option<u64>,
    task: Option<&str>,
    flow: Option<&[String]>,
    confidence: Option<u8>,
) -> Result<()> {
    let mut state = read_state(palbox);

    if let Some(t) = task { state.task = Some(t.to_string()); }
    if let Some(f) = flow { state.flow = Some(f.to_vec()); }
    if let Some(c) = confidence { state.confidence = Some(c); }
    state.updated = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(s) = state.skills.iter_mut().find(|s| s.skill == skill) {
        s.status = status.to_string();
        if status == "inprogress" {
            s.started = Some(chrono::Local::now().format("%H:%M:%S").to_string());
            s.duration_ms = None;
        }
        if let Some(d) = duration_ms { s.duration_ms = Some(d); }
        if let Some(m) = message { s.message = Some(m.to_string()); }
    }

    write_state(palbox, &state)
}

// ── Session recording ───────────────────────────────────────────

/// Record session to .palbox/history/<date-task>.md
pub fn record_session(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("history");
    std::fs::create_dir_all(&dir)?;

    let name: String = task
        .chars()
        .take(50)
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();

    let filename = format!("{}-{}.md", chrono::Local::now().format("%Y-%m-%d"), name.trim_matches('-'));
    let path = dir.join(&filename);
    std::fs::write(&path, content)?;
    Ok(path)
}

// ── Docs syncing (Panthalus enhancement) ─────────────────────────

/// After a task completes, scan the project and update .palbox/ docs.
/// Patches architecture.md with new components/files, checks for database
/// schema changes, and updates flow documentation.
pub fn sync_docs(project_root: &Path, task: &str) -> Result<String> {
    let palbox = project_root.join(".palbox");
    std::fs::create_dir_all(&palbox)?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M");
    let mut report = String::new();

    // ── 1. Architecture.md ──────────────────────────────
    let arch_path = palbox.join("architecture.md");
    let scan = scan_project_files(project_root);
    report.push_str(&format!("  Scanned: {} dirs, {} new files\n", scan.dir_count, scan.new_files.len()));

    if arch_path.exists() {
        let mut existing = std::fs::read_to_string(&arch_path)?;

        // Append new components section
        if !scan.new_files.is_empty() {
            let section = format!(
                "\n## Update: {}\n\n**Task:** {}\n**Date:** {}\n\n### Files created/modified\n\n{}\n",
                now,
                task,
                now,
                scan.new_files.iter().map(|f| format!("- `{}`", f)).collect::<Vec<_>>().join("\n")
            );
            existing.push_str(&section);
            report.push_str("  ✅ Patched architecture.md\n");
        }
        std::fs::write(&arch_path, existing)?;
    } else {
        // Create initial architecture.md
        let arch = format!(
            "# Architecture\n\n**Last updated:** {}\n\n## Overview\n\n{}\n\n## Decisions\n\n### ADR-001\n\n**Status:** Accepted\n**Date:** {}\n**Context:** Task: \"{}\"\n\n## Components\n\n{}\n\n## Data Flow\n\n[tba]\n",
            now,
            task,
            now,
            task,
            scan.new_files.iter().map(|f| format!("- `{}`", f)).collect::<Vec<_>>().join("\n")
        );
        std::fs::write(&arch_path, arch)?;
        report.push_str("  ✅ Created architecture.md\n");
    }

    // ── 2. Database.md (if migrations/schemas detected) ──
    if !scan.db_changes.is_empty() {
        let db_path = palbox.join("database.md");
        let section = format!(
            "## Update: {}\n\n**Task:** {}\n\n{}\n\n",
            now,
            task,
            scan.db_changes.iter().map(|c| format!("- {}", c)).collect::<Vec<_>>().join("\n")
        );

        if db_path.exists() {
            let mut existing = std::fs::read_to_string(&db_path)?;
            existing.push_str(&section);
            std::fs::write(&db_path, existing)?;
        } else {
            let db = format!("# Database\n\n{}\n", section);
            std::fs::write(&db_path, db)?;
        }
        report.push_str("  ✅ Updated database.md\n");
    }

    // ── 3. Flows/ (if new routes/endpoints detected) ──
    if !scan.new_routes.is_empty() {
        let flows_dir = palbox.join("flows");
        std::fs::create_dir_all(&flows_dir)?;
        let flow_content = format!("# Flow: {}\n\n**Date:** {}\n\n## Routes\n\n{}\n",
            task,
            now,
            scan.new_routes.iter().map(|r| format!("- `{}`", r)).collect::<Vec<_>>().join("\n")
        );
        let flow_name: String = task.chars().take(40)
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect();
        let flow_path = flows_dir.join(format!("{}.md", flow_name.trim_matches('-')));
        std::fs::write(&flow_path, flow_content)?;
        report.push_str(&format!("  ✅ Created flow: {}\n", flow_path.display()));
    }

    Ok(report)
}

#[derive(Default)]
struct ProjectScan {
    dir_count: usize,
    new_files: Vec<String>,
    db_changes: Vec<String>,
    new_routes: Vec<String>,
}

/// Lightweight project scanner — walks the tree (excluding node_modules,
/// target, .git, .palbox) and classifies files by type.
fn scan_project_files(root: &Path) -> ProjectScan {
    let mut scan = ProjectScan::default();
    let skip_dirs = ["node_modules", "target", ".git", ".palbox", "__pycache__", ".venv", "dist", "build", ".next"];

    // Walk dirs (only 2 levels for performance)
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if skip_dirs.contains(&name) { continue; }

            if path.is_dir() {
                scan.dir_count += 1;
                // Shallow scan inside dir
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for e in sub.flatten() {
                        let fname = e.file_name().to_string_lossy().to_string();
                        if e.path().is_dir() && !skip_dirs.contains(&fname.as_str()) {
                            scan.dir_count += 1;
                            // Second level
                            if let Ok(sub2) = std::fs::read_dir(e.path()) {
                                for e2 in sub2.flatten() {
                                    classify_file(e2.path(), &mut scan);
                                }
                            }
                        } else {
                            classify_file(e.path(), &mut scan);
                        }
                    }
                }
            } else {
                classify_file(path, &mut scan);
            }
        }
    }

    scan
}

fn classify_file(path: PathBuf, scan: &mut ProjectScan) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let display = path.strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(&path)
        .display()
        .to_string();

    // Detect schema/migration files
    if ext == "sql"
        || name.contains("migration")
        || name.contains("schema")
        || name.contains(".prisma")
    {
        scan.db_changes.push(display.clone());
    }
    // Detect route/controller files
    if name.contains("route")
        || name.contains("controller")
        || name.contains("handler")
        || name == "main.rs"
        || name == "main.py"
        || name == "server.ts"
        || name == "app.ts"
    {
        scan.new_routes.push(display.clone());
    }
    // Track all source files
    if matches!(
        ext,
        "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "sql" | "toml" | "yaml" | "yml" | "json"
    ) {
        scan.new_files.push(display);
    }
}
