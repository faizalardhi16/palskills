//! Dispatch (Anubis) — execute tasks by spawning AI coding agents.
//! Prefers Codex CLI, falls back to printing prompt for manual use.
//! Injects SOLID principles into every dispatch.

use std::path::Path;
use anyhow::Result;

/// Execute a task by dispatching to an AI agent (Codex CLI preferred).
pub fn execute(project_root: &Path, enriched_task: &str) -> Result<()> {
    let cwd = project_root.to_path_buf();

    // Try Codex CLI first
    match std::process::Command::new("codex")
        .args(["exec", "--cd", &cwd.to_string_lossy(), enriched_task])
        .spawn()
    {
        Ok(mut child) => {
            log::info!("🚀 Dispatched to Codex CLI (pid: {})", child.id());
            let status = child.wait()?;
            if status.success() {
                log::info!("✅ Codex completed successfully");
            } else {
                log::warn!("⚠  Codex exited with: {}", status);
            }
        }
        Err(_) => {
            // Fallback: print enriched prompt for manual use
            log::info!("📋 Codex CLI not found. Manual prompt:");
            log::info!("---");
            log::info!("Project: {}", cwd.display());
            log::info!("{}", enriched_task);
            log::info!("---");
        }
    }

    Ok(())
}
