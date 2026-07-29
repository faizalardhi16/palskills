//! Generator — save outputs to .palbox/ as markdown files.

use std::path::{Path, PathBuf};
use anyhow::Result;

/// Create a safe filename from task description.
fn safe_name(task: &str) -> String {
    let name = task
        .chars()
        .take(50)
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>();
    format!("{}-{}", chrono::Local::now().format("%Y-%m-%d"), name.trim_matches('-'))
}

/// Save plan to .palbox/plans/<name>.md
pub fn save_plan(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("plans");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Save PRD to .palbox/prds/<name>.md
pub fn save_prd(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("prds");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Save architecture document.
pub fn save_architecture(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("architectures");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Save schema document.
pub fn save_schema(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("schemas");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Save component tree document.
pub fn save_component(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("components");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Record session to .palbox/history/<name>.md
pub fn record_session(project_root: &Path, task: &str, content: &str) -> Result<PathBuf> {
    let dir = project_root.join(".palbox").join("history");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", safe_name(task)));
    std::fs::write(&path, content)?;
    Ok(path)
}
