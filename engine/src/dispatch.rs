//! Dispatch (Anubis) — SOLID discipline gate.
//!
//! Returns a SOLID-wrapped contract that the main agent must follow.
//! No subprocess — the agent reads the contract and executes with
//! established project conventions + SOLID enforced by the prompt.
//!
//! This is NOT a code runner. It's a discipline gate: the agent can't
//! proceed without reading the constraints first. This prevents god
//! classes, magic numbers, code duplication, and other AI code smells.

use std::path::Path;
use serde::Serialize;

use crate::cbm_bridge;
use crate::palbox_context;

const SOLID_PRINCIPLES: &str = r#"## SOLID Principles (MANDATORY)

1. **Single Responsibility** — each module/class/function has ONE reason to change.
   Split god classes into focused units. Each file should do exactly one thing well.
2. **Open-Closed** — extend behavior via composition/plugins, never modify existing code.
   Add new files for new features; don't cram into existing ones.
3. **Liskov Substitution** — subtypes must be substitutable for their base types.
   If it extends something, it must honor the full contract.
4. **Interface Segregation** — no client should depend on methods it doesn't use.
   Keep interfaces small and focused. Split fat interfaces.
5. **Dependency Inversion** — depend on abstractions (traits/interfaces), not concretions.
   Inject dependencies; don't hardcode implementations.

## Code Quality (MANDATORY)

- **Zero duplication** — extract shared logic, never copy-paste
- **No magic numbers** — all constants named and documented
- **Meaningful names** — variables, functions, files tell their purpose
- **YAGNI** — only build what's needed now
- **Self-documenting code** — comments explain WHY, not WHAT
- **Error handling** — every fallible operation has explicit error handling
- **Tests alongside** — new code requires tests (TDD: RED → GREEN → REFACTOR)"#;

#[derive(Debug, Serialize)]
pub struct SolidityContract {
    pub task: String,
    pub context: DispatchContext,
    pub constraints: String,
    pub gate: String, // instruction for the agent
}

#[derive(Debug, Serialize)]
pub struct DispatchContext {
    pub symbols: Vec<String>,
    pub files: Vec<String>,
    pub source: String,
    pub architecture: Option<String>,
    pub database: Option<String>,
    pub recent_flows: Vec<String>,
}

/// Generate SOLID contract for the main AI agent.
/// Reads CBM + .palbox/ context, wraps in SOLID constraints.
/// Fast-path when no CBM: skips per-keyword grep, uses light file listing.
pub fn generate_contract(project_root: &Path, task: &str) -> anyhow::Result<SolidityContract> {
    // Fast-path: check CBM availability
    let cbm_available = cbm_bridge::check_available(project_root).unwrap_or(false);

    let cbm = if cbm_available {
        cbm_bridge::get_context(project_root, task).unwrap_or_default()
    } else {
        // No CBM — light file listing (same fast-path as scan_context)
        log::info!("⚡ No CBM — dispatch fast-path: listing files only");
        cbm_bridge::CbmContext {
            available: false,
            symbols: vec![],
            callers: vec![],
            architecture: None,
            files: cbm_bridge::quick_file_listing(project_root, task),
            source: "fast-scan".to_string(),
        }
    };
    let docs = palbox_context::read_docs(project_root, task);

    let symbols: Vec<String> = cbm.symbols.iter().map(|s| s.name.clone()).collect();
    let files = cbm.files.clone();
    let source = cbm.source.clone();

    let context = DispatchContext {
        symbols: symbols.clone(),
        files: files.clone(),
        source: source.clone(),
        architecture: docs.architecture_summary,
        database: docs.database_summary,
        recent_flows: docs.recent_flows,
    };

    let gate = format!(
        "🎯 EXECUTE the task below. The SOLID constraints and context are MANDATORY — \
         you MUST apply every principle. No shortcuts, no copy-paste, no god classes. \
         Context shows existing symbols ({sym_count}) and files ({file_count}) from {src} — \
         extend, don't duplicate. Architecture docs: {arch_count}. Review .palbox/ for past decisions.",
        sym_count = symbols.len(),
        file_count = files.len(),
        src = source,
        arch_count = if docs.docs_found > 0 { "loaded" } else { "none" }
    );

    Ok(SolidityContract {
        task: task.to_string(),
        context,
        constraints: SOLID_PRINCIPLES.to_string(),
        gate,
    })
}
