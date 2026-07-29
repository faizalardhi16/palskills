//! Generator — persist pipeline state and session history.
//! State.md is the single source of truth for dashboard display.

use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── State.md types ──────────────────────────────────────────────
// Only 5 skills: orchestrate, scan_context, dispatch, run_tests, record_session

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
