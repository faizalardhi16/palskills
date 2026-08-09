//! Planner (Jetdragon) — group multi-issue requests and generate one plan per group.
//!
//! Engine-side counterpart of the Jetdragon skill: deterministic grouping by
//! dependency + structured plan files in .palbox/plans/. The agent (LLM) fills
//! the reasoning; the engine provides the structure, ordering, and CBM impact.
//!
//! Grouping rules (mirrors skills/jetdragon/SKILL.md Plan Splitting):
//!   1. Suspected shared root cause → ONE group
//!   2. Same module / state machine → ONE group
//!   3. Truly independent → separate groups
//!   4. Root-cause candidates isolated and ordered FIRST

use std::path::{Path, PathBuf};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cbm_bridge;

/// One issue from the user's request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanIssue {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub detail: String,
}

/// A generated group of issues + its plan file.
#[derive(Debug, Serialize)]
pub struct PlanGroup {
    pub name: String,
    pub rationale: String,
    pub issues: Vec<String>,
    pub priority: u8,
    pub plan_path: String,
    pub impact: Vec<String>,
}

/// Result of a multi-issue planning run.
#[derive(Debug, Serialize)]
pub struct PlanOutput {
    pub groups: Vec<PlanGroup>,
    pub summary: String,
    pub confidence: u8,
}

/// Keyword → group-name classifier for heuristic grouping.
/// Deterministic and fast (no LLM) — the agent can refine after.
fn classify(title: &str) -> (String, String) {
    let t = title.to_lowercase();

    // Root-cause candidates: infra / timeout / performance — plan FIRST
    if t.contains("timeout") || t.contains("500") || t.contains("502")
        || t.contains("connection") || t.contains("hang") || t.contains("slow")
    {
        return ("infra".into(), "Root-cause candidate (infra/timeout) — plan first, may fix dependent issues".into());
    }
    // Status / state machine logic (LCL, SI, drafter/exim status)
    if t.contains("status") || t.contains("sudah lengkap") || t.contains("belum lengkap")
        || t.contains("lcl") || t.contains("blocker") || t.contains("state")
    {
        return ("status-logic".into(), "Shared status/state machine — keep changes consistent across entities".into());
    }
    // Submit / data flow between modules
    if t.contains("submit") || t.contains("dikirim") || t.contains("exim")
        || t.contains("notif") || t.contains("email") || t.contains("verif")
    {
        return ("submit-flow".into(), "Shared submit/data-flow path — likely one root cause".into());
    }
    // Document / file upload handling
    if t.contains("upload") || t.contains("download") || t.contains("dokumen")
        || t.contains("document") || t.contains("delete") || t.contains("hapus")
    {
        return ("document".into(), "Document/file handling — independent".into());
    }
    ("independent".into(), "Independent issue — can be worked separately".into())
}

/// Group issues by dependency. Order: infra → status-logic → submit-flow → document → independent.
fn group_issues(issues: &[PlanIssue]) -> Vec<(String, String, Vec<&PlanIssue>)> {
    let priority = |g: &str| match g {
        "infra" => 0,
        "status-logic" => 1,
        "submit-flow" => 2,
        "document" => 3,
        _ => 4,
    };

    let mut buckets: Vec<(String, String, Vec<&PlanIssue>)> = vec![];
    for issue in issues {
        let (group, rationale) = classify(&issue.title);
        if let Some(b) = buckets.iter_mut().find(|(g, _, _)| *g == group) {
            b.2.push(issue);
        } else {
            buckets.push((group, rationale, vec![issue]));
        }
    }
    buckets.sort_by_key(|(g, _, _)| priority(g));
    buckets
}

/// Generate advisory plan content for one group.
fn plan_content(
    root: &Path,
    task_hint: &str,
    issues: &[&PlanIssue],
    rationale: &str,
    impact: &[String],
) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut out = String::new();
    out.push_str(&format!("# Plan: {}\n\n", task_hint));
    out.push_str(&format!("**Date:** {}\n", now));
    out.push_str(&format!("**Group rationale:** {}\n\n", rationale));
    out.push_str("## Issues\n\n");
    for i in issues {
        out.push_str(&format!("- **{}** — {}\n", i.id, i.title));
        if !i.detail.is_empty() {
            out.push_str(&format!("  - {}\n", i.detail));
        }
    }
    out.push('\n');
    out.push_str("## Impact Analysis (CBM)\n\n");
    if impact.is_empty() {
        out.push_str("- ⚠️ No CBM impact available — verify during development\n");
    } else {
        for f in impact {
            out.push_str(&format!("- `{}`\n", f));
        }
    }
    out.push_str("\n## Tasks (ordered)\n\n");
    let mut n = 1;
    for i in issues {
        out.push_str(&format!("{}. **{}** — {}\n", n, i.id, i.title));
        out.push_str(&format!("   - What: {}\n", i.detail));
        out.push_str("   - Files to touch: [tba — from scan_context]\n");
        out.push_str("   - Verification: [tba]\n");
        n += 1;
    }
    out.push_str("\n## Open Questions\n\n- [tba]\n");
    out.push_str("\n---\n> 📋 Auto-generated advisory plan. Agent proceeds immediately — no blocking \"Gas\" gate.\n");
    out
}

/// Run the planner: group issues, query CBM impact, write one plan file per group.
pub fn plan_multi(root: &Path, issues: Vec<PlanIssue>) -> anyhow::Result<PlanOutput> {
    let groups = group_issues(&issues);
    let mut output_groups = vec![];
    let mut plan_dir = root.join(".palbox").join("plans");
    std::fs::create_dir_all(&plan_dir)?;

    let date = chrono::Local::now().format("%Y-%m-%d");
    for (name, rationale, group_issues) in &groups {
        // CBM impact: search for each issue's keywords
        let mut impact: Vec<String> = vec![];
        let keywords: Vec<&str> = group_issues
            .iter()
            .flat_map(|i| {
                i.title
                    .split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '/' || c == '.')
                    .filter(|w| w.len() > 2)
            })
            .collect();
        let (_, files, source) = cbm_bridge::smart_search(root, &keywords);
        if source == "cbm" {
            impact = files;
        }

        let title_hint: String = group_issues
            .first()
            .map(|i| i.title.chars().take(40).collect())
            .unwrap_or_else(|| name.clone());

        let content = plan_content(root, &title_hint, group_issues, rationale, &impact);

        let filename = format!("{}-{}.md", date, name);
        let path = plan_dir.join(&filename);
        std::fs::write(&path, &content)?;

        output_groups.push(PlanGroup {
            name: name.clone(),
            rationale: rationale.clone(),
            issues: group_issues.iter().map(|i| i.id.clone()).collect(),
            priority: groups.iter().position(|(g, _, _)| g == name).unwrap_or(99) as u8 + 1,
            plan_path: path.display().to_string(),
            impact,
        });
    }

    let summary = format!(
        "{} groups generated → {}",
        output_groups.len(),
        output_groups
            .iter()
            .map(|g| format!("{}: {}", g.priority, g.name))
            .collect::<Vec<_>>()
            .join(" | ")
    );

    Ok(PlanOutput {
        groups: output_groups,
        summary,
        confidence: if cbm_bridge::check_available(root).unwrap_or(false) { 85 } else { 70 },
    })
}
