//! CBM Bridge — query codebase memory graph (SQLite) for project structure.

use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CbmContext {
    pub available: bool,
    pub symbols: Vec<SymbolInfo>,
    pub callers: Vec<SymbolInfo>,
    pub architecture: Option<String>,
}

/// Check if CBM index.db exists in project root.
pub fn check_available(project_root: &Path) -> anyhow::Result<bool> {
    let db_path = project_root.join("index.db");
    Ok(db_path.exists())
}

/// Open CBM and search for symbols matching query terms.
pub fn search_symbols(project_root: &Path, query: &str, kind: Option<&str>) -> anyhow::Result<Vec<SymbolInfo>> {
    let db_path = project_root.join("index.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    let mut sql = "SELECT id, name, kind, file_path, line_start, signature FROM symbols WHERE name LIKE ?1".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(format!("%{}%", query))];

    if let Some(k) = kind {
        sql.push_str(" AND kind = ?2");
        params.push(Box::new(k.to_string()));
    }
    sql.push_str(" ORDER BY name LIMIT 20");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            Ok(SymbolInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                file_path: row.get(3)?,
                line_start: row.get::<_, i64>(4)? as usize,
                signature: row.get(5)?,
            })
        },
    )?;

    let mut symbols = Vec::new();
    for row in rows {
        symbols.push(row?);
    }
    Ok(symbols)
}

/// Trace callers of a symbol (who calls this function).
pub fn trace_callers(project_root: &Path, symbol_id: &str) -> anyhow::Result<Vec<SymbolInfo>> {
    let db_path = project_root.join("index.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, s.file_path, s.line_start, s.signature
         FROM edges e JOIN symbols s ON s.id = e.source_id
         WHERE e.target_id = ?1 AND e.kind = 'calls'
         ORDER BY s.name LIMIT 20"
    )?;

    let rows = stmt.query_map([symbol_id], |row| {
        Ok(SymbolInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: row.get(2)?,
            file_path: row.get(3)?,
            line_start: row.get::<_, i64>(4)? as usize,
            signature: row.get(5)?,
        })
    })?;

    let mut symbols = Vec::new();
    for row in rows {
        symbols.push(row?);
    }
    Ok(symbols)
}

/// Get full CBM context: available symbols + architecture overview.
pub fn get_context(project_root: &Path, prompt: &str) -> anyhow::Result<CbmContext> {
    let available = check_available(project_root)?;
    if !available {
        return Ok(CbmContext { available: false, symbols: vec![], callers: vec![], architecture: None });
    }

    let keywords: Vec<&str> = prompt
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|w| w.len() > 3)
        .collect();

    let mut all_symbols = Vec::new();
    for kw in &keywords {
        if let Ok(mut s) = search_symbols(project_root, kw, None) {
            all_symbols.append(&mut s);
        }
    }

    // Get architecture overview
    let db_path = project_root.join("index.db");
    let architecture = if db_path.exists() {
        let conn = rusqlite::Connection::open(&db_path).ok();
        conn.and_then(|c| {
            c.query_row("SELECT value FROM meta WHERE key = 'project_root'", [], |r| {
                r.get::<_, String>(0)
            }).ok()
        })
    } else {
        None
    };

    let token_estimate = all_symbols.len() * 100;
    if !all_symbols.is_empty() {
        log::info!("📦 CBM: {} symbols, ~{} tokens saved", all_symbols.len(), token_estimate);
    }

    Ok(CbmContext {
        available: true,
        symbols: all_symbols,
        callers: vec![],
        architecture,
    })
}
