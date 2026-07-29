//! Palbox Knowledge Graph — indexes .palbox/*.md into SQLite + FTS5.
//!
//! Canonical: .palbox/*.md (human-readable)
//! Query: .palbox/graph.db (agent-readable, low-latency)

use rusqlite::Connection;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub file_path: String,
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResult {
    pub seeds: Vec<Node>,
    pub neighbors: Vec<(Node, String)>,
    pub total_tokens_saved: usize,
}

/// Bootstrap .palbox/ directory with README, architecture, methods, and subdirs.
pub fn bootstrap(project_root: &Path) -> anyhow::Result<()> {
    let palbox = project_root.join(".palbox");
    if palbox.exists() {
        log::info!("⚠  .palbox/ already exists. Skipping bootstrap.");
        return index(&palbox);
    }

    std::fs::create_dir_all(palbox.join("flows"))?;
    std::fs::create_dir_all(palbox.join("history"))?;
    std::fs::create_dir_all(palbox.join("plans"))?;
    std::fs::create_dir_all(palbox.join("prds"))?;
    std::fs::create_dir_all(palbox.join("architectures"))?;
    std::fs::create_dir_all(palbox.join("schemas"))?;
    std::fs::create_dir_all(palbox.join("components"))?;

    let project_name = project_root.file_name()
        .unwrap_or_default()
        .to_string_lossy();

    // README.md
    std::fs::write(palbox.join("README.md"), format!(
        "# {}\n**Bootstrapped:** {}\n\n## Tech Stack\n- [detected during scan]\n\n## Quick Start\n[to be filled]\n",
        project_name, chrono::Local::now().format("%Y-%m-%d")
    ))?;

    // architecture.md
    std::fs::write(palbox.join("architecture.md"), format!(
        "# Architecture\n**Last Updated:** {}\n\n## Folder Structure\n```\n{}```\n\n## Design Patterns\n[tba]\n\n## Key Modules\n[tba]\n",
        chrono::Local::now().format("%Y-%m-%d"), project_name
    ))?;

    // methods.md
    std::fs::write(palbox.join("methods.md"), format!(
        "# Development Methods\n**Last Updated:** {}\n\n## Coding Conventions\n- SOLID principles\n- Single Responsibility Pattern\n- English only\n\n## Testing\n[tba]\n\n## Git Workflow\n[tba]\n",
        chrono::Local::now().format("%Y-%m-%d")
    ))?;

    log::info!("✅ .palbox/ bootstrapped");
    index(&palbox)
}

/// Index all .md files in .palbox/ into SQLite knowledge graph.
pub fn index(palbox: &Path) -> anyhow::Result<()> {
    let db_path = palbox.join("graph.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            title TEXT,
            file_path TEXT NOT NULL,
            summary TEXT,
            content TEXT,
            created_at TEXT,
            updated_at TEXT
        );
        CREATE TABLE IF NOT EXISTS edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id TEXT NOT NULL REFERENCES nodes(id),
            target_id TEXT NOT NULL REFERENCES nodes(id),
            kind TEXT NOT NULL,
            context TEXT
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(title, summary, content);
    ")?;

    // Walk all .md files
    for entry in walkdir::WalkDir::new(palbox)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let path = entry.path();
        let relative = path.strip_prefix(palbox).unwrap_or(path);
        let content = std::fs::read_to_string(path).unwrap_or_default();

        // Determine node kind from parent directory
        let kind = if let Some(parent) = relative.parent() {
            let p = parent.to_string_lossy();
            if p.is_empty() || p == "." { "root".to_string() }
            else if p.contains("flows") { "flow".to_string() }
            else if p.contains("history") { "history".to_string() }
            else if p.contains("plans") { "plan".to_string() }
            else if p.contains("prds") { "prd".to_string() }
            else if p.contains("architectures") { "architecture".to_string() }
            else if p.contains("schemas") { "schema".to_string() }
            else if p.contains("components") { "component".to_string() }
            else { "unknown".to_string() }
        } else {
            "root".to_string()
        };

        let id = relative.with_extension("").to_string_lossy().replace('\\', "/");
        let title = content.lines().next()
            .unwrap_or("Untitled")
            .trim_start_matches("# ")
            .to_string();
        let summary = content.lines().take(2).last()
            .unwrap_or("")
            .to_string();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT OR REPLACE INTO nodes (id, kind, title, file_path, summary, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, kind, title, relative.to_string_lossy(), summary, content, now, now],
        )?;

        // Also insert into FTS
        conn.execute(
            "INSERT OR REPLACE INTO nodes_fts(rowid, title, summary, content) VALUES (
                (SELECT rowid FROM nodes WHERE id = ?1),
                ?2, ?3, ?4
            )",
            rusqlite::params![id, title, summary, content],
        )?;

        // Extract [[wikilinks]] to create edges
        let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for cap in re.captures_iter(&content) {
            let target = cap[1].to_string();
            // Skip self-references
            if target != id {
                conn.execute(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, kind, context) VALUES (?1, ?2, 'references', 'wikilink')",
                    rusqlite::params![id, target],
                )?;
            }
        }
    }

    let count = conn.query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get::<_, i64>(0))?;
    let ecount = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get::<_, i64>(0))?;
    log::info!("📊 Indexed {} nodes, {} edges → {}", count, ecount, db_path.display());
    Ok(())
}

/// Open or create the graph database at the given palbox path.
pub fn open(palbox: &Path) -> anyhow::Result<Connection> {
    let db_path = palbox.join("graph.db");
    if !db_path.exists() {
        index(palbox)?;
    }
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    Ok(conn)
}

/// Search for nodes matching a query (FTS5).
pub fn search(conn: &Connection, query: &str) -> anyhow::Result<Vec<Node>> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.kind, n.title, n.file_path, n.summary, n.created_at, n.updated_at
         FROM nodes_fts f JOIN nodes n ON f.rowid = n.rowid
         WHERE nodes_fts MATCH ?1 ORDER BY rank LIMIT 20"
    )?;
    let rows = stmt.query_map([query], |row| {
        Ok(Node {
            id: row.get(0)?,
            kind: row.get(1)?,
            title: row.get(2)?,
            file_path: row.get(3)?,
            summary: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}

/// Get neighbor nodes (1-hop) for a set of seed IDs.
pub fn get_neighbors(conn: &Connection, seed_ids: &[String]) -> anyhow::Result<Vec<(Node, String)>> {
    if seed_ids.is_empty() { return Ok(vec![]); }
    let placeholders = seed_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT n.id, n.kind, n.title, n.file_path, n.summary, n.created_at, n.updated_at, e.kind
         FROM edges e JOIN nodes n ON n.id = e.target_id
         WHERE e.source_id IN ({}) LIMIT 50",
        placeholders
    );

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = seed_ids.iter()
        .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::types::ToSql>)
        .collect();

    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            Ok((
                Node {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    title: row.get(2)?,
                    file_path: row.get(3)?,
                    summary: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                },
                row.get::<_, String>(7)?
            ))
        },
    )?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Unified context scan: search palbox graph and return seeds + neighbors.
pub fn scan_context(conn: &Connection, prompt: &str) -> anyhow::Result<ContextResult> {
    let seeds = search(conn, prompt)?;
    let seed_ids: Vec<String> = seeds.iter().map(|n| n.id.clone()).collect();
    let neighbors = get_neighbors(conn, &seed_ids)?;
    let tokens_saved = seeds.len() * 500 + neighbors.len() * 200; // rough estimate

    Ok(ContextResult {
        seeds,
        neighbors,
        total_tokens_saved: tokens_saved,
    })
}
