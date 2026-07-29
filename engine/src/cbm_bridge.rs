//! CBM Bridge — query codebase memory graph (SQLite) for project structure.
//!
//! Strategy: CBM FIRST for all code discovery. Fallback to grep/file-scan
//! ONLY when CBM is unavailable or returns zero results.

use std::path::Path;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatch {
    pub path: String,
    pub matches: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CbmContext {
    pub available: bool,
    pub symbols: Vec<SymbolInfo>,
    pub callers: Vec<SymbolInfo>,
    pub architecture: Option<String>,
    pub files: Vec<String>,
    pub source: String, // "cbm" | "grep" | "mixed"
}

/// Check if CBM index.db exists in project root.
pub fn check_available(project_root: &Path) -> anyhow::Result<bool> {
    let db_path = project_root.join("index.db");
    Ok(db_path.exists())
}

/// CBM symbol search. Always preferred over grep.
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
    sql.push_str(" ORDER BY name LIMIT 50");

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

/// CBM: list all files known to the index.
pub fn list_files(project_root: &Path) -> anyhow::Result<Vec<String>> {
    let db_path = project_root.join("index.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    let mut stmt = conn.prepare("SELECT DISTINCT file_path FROM symbols ORDER BY file_path")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut files = Vec::new();
    for row in rows {
        files.push(row?);
    }
    Ok(files)
}

/// CBM: search for files matching a pattern (by path, not symbol name).
pub fn search_files(project_root: &Path, pattern: &str) -> anyhow::Result<Vec<String>> {
    let db_path = project_root.join("index.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT file_path FROM symbols WHERE file_path LIKE ?1 ORDER BY file_path LIMIT 50"
    )?;
    let rows = stmt.query_map([format!("%{}%", pattern)], |row| row.get::<_, String>(0))?;
    let mut files = Vec::new();
    for row in rows {
        files.push(row?);
    }
    Ok(files)
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

// ── GREP FALLBACK ────────────────────────────────────────────────

/// Grep for files containing a pattern. Used ONLY when CBM is unavailable or returns 0 results.
pub fn grep_files(project_root: &Path, pattern: &str, file_ext: Option<&str>) -> Vec<String> {
    let ext_filter = file_ext.unwrap_or("*");
    let cmd = if cfg!(windows) {
        format!("rg -l --max-count 1 \"{}\" --glob \"*.{}\" src 2>nul", pattern, ext_filter)
    } else {
        format!("rg -l --max-count 1 \"{}\" --glob \"*.{}\" 2>/dev/null || grep -rl \"{}\" --include=\"*.{}\" 2>/dev/null", pattern, ext_filter, pattern, ext_filter)
    };

    if let Ok(out) = Command::new("sh").arg("-c").arg(&cmd).current_dir(project_root).output() {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        vec![]
    }
}

/// Find files by name pattern. CBM first, then find/walkdir fallback.
pub fn find_files(project_root: &Path, name_pattern: &str) -> Vec<String> {
    // CBM first
    if let Ok(cbm_files) = search_files(project_root, name_pattern) {
        if !cbm_files.is_empty() {
            return cbm_files;
        }
    }

    // Fallback: walkdir
    let mut files = Vec::new();
    if let Ok(entries) = walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy().contains(name_pattern)
        })
        .take(50)
        .try_fold(Vec::new(), |mut acc, e| {
            acc.push(e.path().to_string_lossy().to_string());
            Ok::<_, walkdir::Error>(acc)
        })
    {
        files = entries;
    }

    files
}

/// Keyword search: CBM symbols first, grep fallback if CBM returns nothing.
pub fn smart_search(project_root: &Path, keywords: &[&str]) -> (Vec<SymbolInfo>, Vec<String>, String) {
    let cbm_available = check_available(project_root).unwrap_or(false);

    if cbm_available {
        let mut all_symbols = Vec::new();
        let mut all_files = Vec::new();

        for kw in keywords {
            if let Ok(syms) = search_symbols(project_root, kw, None) {
                for s in &syms {
                    if !all_files.contains(&s.file_path) {
                        all_files.push(s.file_path.clone());
                    }
                }
                all_symbols.extend(syms);
            }
        }

        // Deduplicate
        all_symbols.sort_by(|a, b| a.name.cmp(&b.name));
        all_symbols.dedup_by(|a, b| a.id == b.id);
        all_files.sort();
        all_files.dedup();

        if !all_symbols.is_empty() {
            log::info!("📦 CBM: {} symbols in {} files", all_symbols.len(), all_files.len());
            return (all_symbols, all_files, "cbm".to_string());
        }
    }

    // CBM returned nothing or unavailable → grep fallback
    log::info!("🔍 CBM miss — falling back to grep...");
    let mut all_files = Vec::new();
    for kw in keywords {
        let files = grep_files(project_root, kw, None);
        for f in files {
            if !all_files.contains(&f) {
                all_files.push(f);
            }
        }
    }

    log::info!("   Grep found {} files", all_files.len());
    (vec![], all_files, "grep".to_string())
}

// ── UNIFIED CONTEXT BUILDER ─────────────────────────────────────

/// Get full context: CBM symbols + files. CBM first, grep fallback.
pub fn get_context(project_root: &Path, prompt: &str) -> anyhow::Result<CbmContext> {
    let keywords: Vec<&str> = prompt
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '/' || c == '.')
        .filter(|w| w.len() > 2)
        .collect();

    let (symbols, files, source) = smart_search(project_root, &keywords);

    // Architecture overview from CBM
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

    let token_estimate = symbols.len() * 100 + files.len() * 50;
    if token_estimate > 0 {
        log::info!("📦 Context: {} symbols, {} files → ~{} tokens saved (source: {})",
            symbols.len(), files.len(), token_estimate, source);
    }

    let available = !source.is_empty();

    Ok(CbmContext {
        available,
        symbols,
        callers: vec![],
        architecture,
        files,
        source,
    })
}
