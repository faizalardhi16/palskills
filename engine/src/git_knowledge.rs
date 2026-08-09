//! Git Knowledge — auto-capture commit metadata into .palbox/pending/.
//!
//! Two-layer knowledge recording (design):
//!   Layer 1 (automatic): git hook / sync-git captures METADATA per commit
//!     (files changed, commit message, diff stat) → .palbox/pending/*.json
//!   Layer 2 (rich): record_session reads pending/, agent fills WHY
//!     (decisions, lessons, api) — never loses the WHAT even if agent forgets.
//!
//! scan_context flags unrecorded pending so the agent is guided to consolidate.

use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub when: String,
    pub files: Vec<String>,
    pub recorded: bool,
}

/// Read all pending commits from .palbox/pending/.
pub fn read_pending(project_root: &Path) -> Vec<PendingCommit> {
    let dir = project_root.join(".palbox").join("pending");
    let mut out = vec![];
    if !dir.exists() {
        return out;
    }
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if let Ok(c) = serde_json::from_str::<PendingCommit>(&content) {
                        out.push(c);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.when.cmp(&a.when));
    out
}

/// Unrecorded pending commits (not yet consolidated by record_session).
pub fn read_unrecorded(project_root: &Path) -> Vec<PendingCommit> {
    read_pending(project_root)
        .into_iter()
        .filter(|c| !c.recorded)
        .collect()
}

/// Mark pending commits as recorded (called by record_session).
pub fn mark_recorded(project_root: &Path, hashes: &[String]) -> Result<usize> {
    let dir = project_root.join(".palbox").join("pending");
    let mut count = 0;
    if !dir.exists() {
        return Ok(0);
    }
    for e in std::fs::read_dir(&dir)? {
        let p = e?.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&p) else { continue };
        let Ok(mut c) = serde_json::from_str::<PendingCommit>(&content) else { continue };
        if hashes.iter().any(|h| h == &c.hash) && !c.recorded {
            c.recorded = true;
            if let Ok(json) = serde_json::to_string_pretty(&c) {
                let _ = std::fs::write(&p, json);
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Run `git log` since a marker (or full history) and capture metadata.
/// Writes one JSON per commit into .palbox/pending/ (idempotent by hash).
pub fn sync_git(project_root: &Path) -> Result<(usize, usize)> {
    let dir = project_root.join(".palbox").join("pending");
    std::fs::create_dir_all(&dir)?;

    // Existing hashes → skip duplicates
    let existing: std::collections::HashSet<String> = read_pending(project_root)
        .into_iter()
        .map(|c| c.hash)
        .collect();

    let output = std::process::Command::new("git")
        .arg("log")
        .arg("--pretty=format:%H%x1f%an%x1f%ad%x1f%s")
        .arg("--date=iso")
        .arg("-n")
        .arg("30")
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        return Ok((0, 0)); // not a git repo or no commits yet
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut added = 0;
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\u{1f}').collect();
        if parts.len() < 4 {
            continue;
        }
        let hash = parts[0].trim().to_string();
        if existing.contains(&hash) {
            continue;
        }
        let author = parts[1].trim().to_string();
        let when = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        // Get changed files for this commit
        let files = changed_files(project_root, &hash);

        let pending = PendingCommit {
            hash: hash.clone(),
            message,
            author,
            when,
            files,
            recorded: false,
        };
        let path = dir.join(format!("{}.json", &hash));
        std::fs::write(&path, serde_json::to_string_pretty(&pending)?)?;
        added += 1;
    }

    Ok((added, existing.len()))
}

/// Files changed in a specific commit (git show --name-only).
fn changed_files(project_root: &Path, hash: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("show")
        .arg("--name-only")
        .arg("--pretty=format:")
        .arg(hash)
        .current_dir(project_root)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    if !output.status.success() {
        return vec![];
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Changed files vs a base ref (default: HEAD) — used by run_tests impact analysis.
pub fn diff_files(project_root: &Path, base: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(base)
        .current_dir(project_root)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    if !output.status.success() {
        return vec![];
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Install a post-commit git hook that calls sync-git metadata capture.
/// The hook is cheap (git log -n 30 + per-commit file listing, <200ms typical).
pub fn install_hook(project_root: &Path, engine_bin: &str) -> Result<PathBuf> {
    let hooks = project_root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks)?;
    let hook_path = hooks.join("post-commit");

    let script = format!(
        r#"#!/bin/sh
# Palskills — auto-capture commit metadata into .palbox/pending/
# Layer 1 of knowledge recording. Layer 2 (rich WHY) happens via record_session.
if command -v "{}" >/dev/null 2>&1; then
  "{}" sync-git --project "{}" >/dev/null 2>&1 || true
fi
"#,
        engine_bin,
        engine_bin,
        project_root.display()
    );

    std::fs::write(&hook_path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }
    Ok(hook_path)
}
