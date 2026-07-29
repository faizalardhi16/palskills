//! Dispatch (Anubis) — execute plans by spawning AI agents.

use std::path::Path;
use anyhow::Result;
use crate::orchestrator::OrchestrationPlan;

/// Execute a build by dispatching to an AI agent (Codex CLI preferred).
pub fn execute(
    project_root: &Path,
    task: &str,
    _plan: &str,
    _ctx: &OrchestrationPlan,
) -> Result<()> {
    let cwd = project_root.to_path_buf();

    // Try Codex CLI first
    let codex_result = std::process::Command::new("codex")
        .args(["exec", "--cd", &cwd.to_string_lossy(), task])
        .spawn();

    match codex_result {
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
            // Fallback: print the enriched prompt for manual use
            log::info!("📋 Codex CLI not found. Here's your prompt:");
            log::info!("---");
            log::info!("Task: {}", task);
            log::info!("Project: {}", cwd.display());
            log::info!("Follow SOLID + SRP. Write tests first. Document APIs.");
            log::info!("---");
            log::info!("Copy this into Cursor / Claude Code / Codex CLI and execute.");
        }
    }

    Ok(())
}
