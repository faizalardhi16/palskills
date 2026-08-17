//! Palskills Engine — CBM-aware MCP orchestration (5 tools).
//!
//! Pipeline: orchestrate → scan_context → dispatch → run_tests → record_session
//! orchestrate detects flow + auto-generates advisory plans for complex tasks.
//! dispatch is a SOLID contract gate (no subprocess — agent main yg eksekusi).
//! record_session syncs docs (.palbox/architecture.md, database.md, flows/)
//! after every completed task — keeping documentation in sync with reality.
//!
//!   palskills-engine init   → bootstrap .palbox/
//!   palskills-engine serve  → MCP server (5 tools) + dashboard on http://localhost:3030

mod server;
mod dashboard;
mod cbm_bridge;
mod orchestrator;
mod dispatch;
mod generator;
mod git_knowledge;
mod palbox_context;
mod planner;
mod templates;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "palskills-engine", about = "Palskills Engine — CBM-aware MCP orchestration")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Bootstrap .palbox/ directory structure
    Init {
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },
    /// Start 6-tool MCP server + dashboard on :3030
    Serve {
        #[arg(short, long)]
        project: Option<PathBuf>,
        /// Root containing .palbox/ — defaults to project (monorepo/nested: point to parent)
        #[arg(long)]
        palbox: Option<PathBuf>,
        #[arg(long, default_value = "index.db")]
        cbm: PathBuf,
    },
    /// Auto-capture git commit metadata into .palbox/pending/ (Layer 1 knowledge)
    SyncGit {
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
        /// Root containing .palbox/ — defaults to project (monorepo: point to parent)
        #[arg(long)]
        palbox: Option<PathBuf>,
    },
    /// Install post-commit git hook for auto-knowledge capture
    InstallHook {
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
        /// Root containing .palbox/ — defaults to project (monorepo: point to parent)
        #[arg(long)]
        palbox: Option<PathBuf>,
    },
}

/// Create minimal .palbox/ directory structure.
/// No SQLite — knowledge lives in .md files (State.md, history/*.md).
fn bootstrap(palbox: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(palbox.join("history"))?;
    std::fs::write(palbox.join("State.md"), "# Pipeline State\n\nNo active pipeline.\n")?;
    log::info!("✅ .palbox/ bootstrapped");
    Ok(())
}

/// Project root = the folder that was opened (CWD), NOT a walk-up discovery.
/// Deliberate: if Cursor is opened at C:/faizal/exon, .palbox lives there.
/// No walk-up — otherwise a stray .palbox in a parent folder hijacks unrelated
/// projects. If CWD has no .palbox, the engine runs in PASSIVE mode
/// (no recording, no knowledge context, no folder creation).
fn detect_project_root() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_dir()?)
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Command::Init { project } => {
            let palbox = project.join(".palbox");
            bootstrap(&palbox)?;
            match templates::generate(&project) {
                Ok(written) => {
                    if written.is_empty() {
                        log::info!("AGENTS.md + .cursorrules already exist — kept as-is");
                    } else {
                        for p in &written {
                            log::info!("✅ Generated {}", p.display());
                        }
                    }
                }
                Err(e) => log::warn!("⚠ Template generation failed: {e}"),
            }
            log::info!("Run 'palskills-engine serve' to start.");
        }
        Command::SyncGit { project, palbox } => {
            let root = project;
            if !root.join(".git").exists() {
                log::error!("Not a git repository: {}", root.display());
                std::process::exit(1);
            }
            let palbox_root = palbox.unwrap_or_else(|| root.clone());
            match git_knowledge::sync_git(&root, &palbox_root) {
                Ok((added, existing)) => {
                    log::info!("✅ Git sync: {} new commits captured, {} already pending", added, existing);
                }
                Err(e) => {
                    log::error!("Git sync failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::InstallHook { project, palbox } => {
            let root = project;
            let palbox_root = palbox.unwrap_or_else(|| root.clone());
            let engine_bin = std::env::current_exe()?.display().to_string();
            match git_knowledge::install_hook(&root, &palbox_root, &engine_bin) {
                Ok(path) => log::info!("✅ post-commit hook installed: {}", path.display()),
                Err(e) => {
                    log::error!("Hook install failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Serve { project, palbox, cbm } => {
            let rt = tokio::runtime::Runtime::new()?;

            // project = CWD (opened folder); palbox defaults to project root.
            // If no .palbox exists → PASSIVE mode: no recording, no context,
            // no folder creation. User must run `init` explicitly to enable.
            let project_root = project.unwrap_or_else(|| detect_project_root().unwrap_or_else(|_| PathBuf::from(".")));
            let palbox_root = palbox.unwrap_or_else(|| project_root.clone());
            let palbox = palbox_root.join(".palbox");
            let palbox_active = palbox.exists();

            if palbox_active {
                log::info!("📦 .palbox ACTIVE at {}", palbox.display());
            } else {
                log::warn!(
                    "⚪ No .palbox at {} — PASSIVE mode (no recording, no context). Run 'palskills-engine init' to enable.",
                    palbox.display()
                );
            }

            log::info!("📁 Project root (CWD-based): {}", project_root.display());

            rt.block_on(async {
                let dash_palbox = palbox.clone();
                tokio::spawn(async {
                    if let Err(e) = dashboard::serve(dash_palbox).await {
                        eprintln!("Dashboard error: {e}");
                    }
                });
                log::info!("🌐 Dashboard: http://localhost:3030");

                if let Err(e) = server::run_server(project_root, palbox, palbox_active, cbm).await {
                    eprintln!("Server error: {e}");
                }
            });
        }
    }

    Ok(())
}
