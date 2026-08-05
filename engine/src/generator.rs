//! Generator — persist pipeline state and session history.
//! State.md is the single source of truth for dashboard display.
//!
//! 5 skills: orchestrate, scan_context, dispatch, run_tests, record_session

use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── State.md types ──────────────────────────────────────────────
// 5 skills: orchestrate → scan_context → dispatch → run_tests → record_session

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
    /// Auto-appended audit trail — every tool call logged here by the engine.
    pub session_log: Vec<String>,
}

const ALL_SKILLS: &[&str] = &[
    "orchestrate",
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
        session_log: vec![],
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
        // Session log lines: "- HH:MM:SS [skill] message"
        if line.starts_with("- ") && line.contains("[") && line.contains("]") {
            state.session_log.push(line.to_string());
        }
    }
    // Cap session log to last 200 entries
    if state.session_log.len() > 200 {
        let keep = state.session_log.len() - 200;
        state.session_log.drain(..keep);
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

    // Session log — audit trail of every tool call
    out.push_str("\n## 📜 Session Log\n\n");
    if state.session_log.is_empty() {
        out.push_str("_No tool calls recorded yet._\n");
    } else {
        for entry in &state.session_log {
            out.push_str(entry);
            out.push('\n');
        }
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

    // Auto-append audit trail — engine-enforced, agent cannot skip
    let icon = match status {
        "done" => "✅",
        "inprogress" => "🔄",
        "error" => "❌",
        _ => "⏳",
    };
    let dur = duration_ms.map_or(String::new(), |d| format!(" ({}ms)", d));
    let task_label = task.map(|t| format!(" · {}", t)).unwrap_or_default();
    let msg = message.unwrap_or("");
    let entry = format!(
        "- {} {} `{}` {} — {}{}{}",
        chrono::Local::now().format("%H:%M:%S"),
        icon,
        skill,
        status,
        msg,
        dur,
        task_label
    );
    state.session_log.push(entry);
    if state.session_log.len() > 200 {
        let keep = state.session_log.len() - 200;
        state.session_log.drain(..keep);
    }

    write_state(palbox, &state)
}

// ── Session recording ───────────────────────────────────────────

/// Structured knowledge captured from a completed task.
/// Written by the AGENT (LLM), stored by the engine — this is what makes
/// .palbox/ a real knowledge base instead of a file list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionKnowledge {
    pub summary: String,
    pub decisions: Vec<String>,
    pub modules: Vec<String>,
    pub lessons: Vec<String>,
    pub api: Vec<String>,
}

/// Record session knowledge to .palbox/history/<date-task>.md
pub fn record_session(project_root: &Path, k: &SessionKnowledge) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("history");
    std::fs::create_dir_all(&dir)?;

    let name: String = k
        .summary
        .chars()
        .take(50)
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();

    let filename = format!("{}-{}.md", chrono::Local::now().format("%Y-%m-%d"), name.trim_matches('-'));
    let path = dir.join(&filename);

    let mut content = String::new();
    content.push_str(&format!(
        "# Session: {}\n**Date:** {}\n\n",
        k.summary,
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    content.push_str(&format!("## Summary\n{}\n\n", k.summary));

    if !k.decisions.is_empty() {
        content.push_str("## Decisions\n");
        for d in &k.decisions {
            content.push_str(&format!("- {}\n", d));
        }
        content.push('\n');
    }
    if !k.modules.is_empty() {
        content.push_str("## Modules\n");
        for m in &k.modules {
            content.push_str(&format!("- {}\n", m));
        }
        content.push('\n');
    }
    if !k.api.is_empty() {
        content.push_str("## API\n");
        for a in &k.api {
            content.push_str(&format!("- `{}`\n", a));
        }
        content.push('\n');
    }
    if !k.lessons.is_empty() {
        content.push_str("## Lessons\n");
        for l in &k.lessons {
            content.push_str(&format!("- {}\n", l));
        }
        content.push('\n');
    }

    std::fs::write(&path, content)?;
    Ok(path)
}

// ── Docs syncing (Panthalus enhancement) ─────────────────────────

/// After a task completes, scan the project and update .palbox/ docs.
/// Patches architecture.md with new components/files, checks for database
/// schema changes, updates flow documentation, AND persists structured
/// knowledge (decisions, modules, api) written by the agent.
pub fn sync_docs(project_root: &Path, task: &str, k: &SessionKnowledge) -> Result<String> {
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

    // ── 4. Decisions.md (ADR log — agent-written knowledge) ──
    if !k.decisions.is_empty() {
        let decisions_path = palbox.join("decisions.md");
        let section = format!(
            "### ADR: {}\n\n**Date:** {}\n**Task:** {}\n\n{}\n\n",
            now,
            now,
            task,
            k.decisions.iter().map(|d| format!("- {}\n", d)).collect::<String>()
        );
        if decisions_path.exists() {
            let mut existing = std::fs::read_to_string(&decisions_path)?;
            existing.push_str(&section);
            std::fs::write(&decisions_path, existing)?;
        } else {
            let decisions = format!("# Decisions (ADR)\n\n{}\n", section);
            std::fs::write(&decisions_path, decisions)?;
        }
        report.push_str(&format!("  ✅ Updated decisions.md ({} ADRs)\n", k.decisions.len()));
    }

    // ── 5. Modules/ (per-module knowledge files) ──
    if !k.modules.is_empty() {
        let modules_dir = palbox.join("modules");
        std::fs::create_dir_all(&modules_dir)?;
        for m in &k.modules {
            // Format: "module_name: description" or "name — description"
            let (name, desc) = match m.split_once(':') {
                Some((n, d)) => (n.trim(), d.trim().to_string()),
                None => (m.as_str(), String::new()),
            };
            let safe: String = name
                .chars()
                .take(30)
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
                .collect();
            let module_path = modules_dir.join(format!("{}.md", safe.trim_matches('-')));
            let content = format!(
                "# Module: {}\n\n**Last updated:** {}\n\n## Purpose\n\n{}\n\n## Source\n\n_Recorded from task: {}_\n",
                name, now, desc, task
            );
            std::fs::write(&module_path, content)?;
        }
        report.push_str(&format!("  ✅ Updated modules/ ({} modules)\n", k.modules.len()));
    }

    // ── 6. API.md (endpoint contracts — agent-written knowledge) ──
    if !k.api.is_empty() {
        let api_path = palbox.join("api.md");
        let section = format!(
            "## Update: {}\n\n**Task:** {}\n\n{}\n\n",
            now,
            task,
            k.api.iter().map(|a| format!("- `{}`\n", a)).collect::<String>()
        );
        if api_path.exists() {
            let mut existing = std::fs::read_to_string(&api_path)?;
            existing.push_str(&section);
            std::fs::write(&api_path, existing)?;
        } else {
            let api = format!("# API Contracts\n\n{}\n", section);
            std::fs::write(&api_path, api)?;
        }
        report.push_str(&format!("  ✅ Updated api.md ({} endpoints)\n", k.api.len()));
    }

    // ── 7. Lessons.md (accumulated lessons) ──
    if !k.lessons.is_empty() {
        let lessons_path = palbox.join("lessons.md");
        let section = format!(
            "## {}\n\n**Task:** {}\n\n{}\n\n",
            now,
            task,
            k.lessons.iter().map(|l| format!("- {}\n", l)).collect::<String>()
        );
        if lessons_path.exists() {
            let mut existing = std::fs::read_to_string(&lessons_path)?;
            existing.push_str(&section);
            std::fs::write(&lessons_path, existing)?;
        } else {
            let lessons = format!("# Lessons Learned\n\n{}\n", section);
            std::fs::write(&lessons_path, lessons)?;
        }
        report.push_str(&format!("  ✅ Updated lessons.md ({} lessons)\n", k.lessons.len()));
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
/// Hard cap at MAX_FILES to prevent timeout on large projects.
const MAX_SCAN_FILES: usize = 300;

fn scan_project_files(root: &Path) -> ProjectScan {
    let mut scan = ProjectScan::default();
    let skip_dirs = ["node_modules", "target", ".git", ".palbox", "__pycache__", ".venv", "dist", "build", ".next", ".turbo", "coverage", "uploads", "public/assets"];

    // Walk dirs (only 2 levels for performance) — bail at MAX_SCAN_FILES
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            if scan.new_files.len() >= MAX_SCAN_FILES { break; }
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if skip_dirs.contains(&name) || name.starts_with('.') { continue; }

            if path.is_dir() {
                scan.dir_count += 1;
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for e in sub.flatten() {
                        if scan.new_files.len() >= MAX_SCAN_FILES { break; }
                        let fname = e.file_name().to_string_lossy().to_string();
                        if e.path().is_dir() && !skip_dirs.contains(&fname.as_str()) && !fname.starts_with('.') {
                            scan.dir_count += 1;
                            // Second level — only classify, don't recurse further
                            if let Ok(sub2) = std::fs::read_dir(e.path()) {
                                for e2 in sub2.flatten() {
                                    if scan.new_files.len() >= MAX_SCAN_FILES { break; }
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

    if scan.new_files.len() >= MAX_SCAN_FILES {
        log::info!("⚡ Scan capped at {} files (MAX_SCAN_FILES)", MAX_SCAN_FILES);
    }

    scan
}

fn classify_file(path: PathBuf, scan: &mut ProjectScan) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Fast skip: only track source/config files
    if !matches!(
        ext,
        "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "sql" | "toml" | "yaml" | "yml" | "json"
    ) {
        return;
    }

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
