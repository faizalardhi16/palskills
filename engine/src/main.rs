//! Palskills Engine — AI development pipeline orchestration.
//!
//! Two modes:
//!   palskills init                      → bootstrap .palbox/ from project
//!   palskills serve                     → start MCP server (11 tools)
//!   palskills plan "build login"        → Jetdragon mode (interactive)
//!   palskills build "build login"       → full pipeline

mod server;
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
        /// Path to .palbox/ (default: ./.palbox)
        #[arg(short, long, default_value = ".palbox")]
        palbox: PathBuf,
        /// Path to CBM index.db (optional)
        #[arg(long, default_value = "index.db")]
        cbm: PathBuf,
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
        Command::Serve { palbox, cbm } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
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
