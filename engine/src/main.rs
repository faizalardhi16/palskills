//! Palskills Engine — AI development pipeline orchestration.
//!
//!   palskills init                      → bootstrap .palbox/
//!   palskills serve                     → MCP server (11 tools)
//!   palskills serve --ui                → MCP + dashboard on http://localhost:3030
//!   palskills plan "build login"        → Jetdragon mode
//!   palskills build "build login"       → full pipeline

mod server;
mod dashboard;
mod palbox_graph;
mod cbm_bridge;
mod orchestrator;
mod planner;
mod generator;
mod dispatch;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "palskills", about = "Palskills Engine — AI development pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Bootstrap .palbox/ — analyze project, create knowledge graph
    Init {
        /// Project root (default: current directory)
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },
    /// Start MCP server — 11 skills as tools for AI agents
    Serve {
        /// Project root directory. Auto-detected if not specified
        #[arg(short, long)]
        project: Option<PathBuf>,
        /// Path to CBM index.db (default: index.db)
        #[arg(long, default_value = "index.db")]
        cbm: PathBuf,
        /// Enable web dashboard on http://localhost:3030
        #[arg(long)]
        ui: bool,
    },
    /// Jetdragon mode: brainstorm → clarify → generate plan
    Plan {
        /// Task description
        task: String,
    },
    /// Full pipeline: context → plan → dispatch → record
    Build {
        /// Task description
        task: String,
        /// Auto-confirm (skip clarifying questions)
        #[arg(short, long)]
        yes: bool,
    },
}

/// Auto-detect project root by walking up from CWD.
/// Looks for .palbox/, package.json, Cargo.toml, go.mod, or index.db.
fn detect_project_root() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut current = cwd.clone();

    loop {
        // Check for project markers
        if current.join(".palbox").exists()
            || current.join("package.json").exists()
            || current.join("Cargo.toml").exists()
            || current.join("go.mod").exists()
            || current.join("index.db").exists()
        {
            log::info!("📍 Detected project root: {}", current.display());
            return Ok(current);
        }

        // Walk up
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }

        // Don't go past root
        if current.parent().is_none() && !current.join("package.json").exists() {
            break;
        }
    }

    // Fallback: use CWD
    log::info!("📍 No project markers found, using CWD: {}", cwd.display());
    Ok(cwd)
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let cli = Cli::parse();

    match cli.command {
        Command::Init { project } => {
            log::info!("🔍 Bootstrapping .palbox/ from {}", project.display());
            palbox_graph::bootstrap(&project)?;
            // Also try to index CBM if available
            if let Err(e) = cbm_bridge::check_available(&project) {
                log::warn!("CBM not available: {}", e);
            }
            log::info!("✅ .palbox/ ready. Run 'palskills serve' to start MCP server.");
        }
        Command::Serve { project, cbm, ui } => {
            let rt = tokio::runtime::Runtime::new()?;

            // Determine project root: explicit flag → auto-detect → CWD
            let project_root = if let Some(p) = project {
                p
            } else {
                detect_project_root()?
            };
            let palbox = project_root.join(".palbox");

            // Auto-bootstrap .palbox/ if missing
            if !palbox.exists() {
                log::info!("🔍 .palbox/ not found — bootstrapping from {}", project_root.display());
                palbox_graph::bootstrap(&project_root)?;
            }

            rt.block_on(async {
                if ui {
                    let dash_palbox = palbox.clone();
                    tokio::spawn(async {
                        if let Err(e) = dashboard::serve(dash_palbox).await {
                            eprintln!("Dashboard error: {}", e);
                        }
                    });
                    log::info!("🌐 Dashboard: http://localhost:3030");
                }
                if let Err(e) = server::run_server(palbox, cbm).await {
                    eprintln!("Server error: {}", e);
                }
            });
        }
        Command::Plan { task } => {
            let cwd = std::env::current_dir()?;
            let plan = planner::generate_plan(&cwd, &task, false)?;
            let path = generator::save_plan(&cwd, &task, &plan)?;
            log::info!("📋 Plan saved: {}", path.display());
        }
        Command::Build { task, yes } => {
            let cwd = std::env::current_dir()?;
            log::info!("🔨 Building: {}", task);
            // 1. Context
            let ctx = orchestrator::analyze(&cwd, &task)?;
            log::info!("   Flow: {:?}", ctx.flow);
            log::info!("   Confidence: {}%", ctx.confidence);
            // 2. Plan
            let plan = planner::generate_plan(&cwd, &task, yes)?;
            // 3. Dispatch
            dispatch::execute(&cwd, &task, &plan, &ctx)?;
            // 4. Record
            generator::record_session(&cwd, &task, &plan)?;
            log::info!("✅ Done.");
        }
    }

    Ok(())
}
