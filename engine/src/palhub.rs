//! PalHub knowledge backend — versi palskills-engine yang record & scan context ke PalHub.
//!
//! `scan_context` baca knowledge dari PalHub (REST search di /api/specialists/:id/knowledge/search),
//! `record_session` nulis session knowledge ke PalHub (POST /api/specialists/:id/knowledge).
//! Fail-open: kalau PalHub unreachable → warn ke stderr, local .palbox tetap jalan.
//!
//! Env:
//!   PALHUB_URL         — default http://127.0.0.1:8787
//!   PALHUB_SPECIALIST  — default 10 (specialist "Engineering" di palhub-web)
//!   PALHUB_DISABLED=1  — matikan PalHub backend (kalau environment tanpa palhub)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PalhubClient {
    pub base_url: String,
    pub specialist_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHit {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub source: Option<String>,
}

impl PalhubClient {
    pub fn from_env() -> Option<Self> {
        if std::env::var("PALHUB_DISABLED").is_ok() {
            return None;
        }
        let base_url = std::env::var("PALHUB_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
        let specialist_id: i64 = std::env::var("PALHUB_SPECIALIST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10); // "Engineering" specialist di palhub-web
        Some(Self { base_url, specialist_id })
    }

    /// scan_context: cari knowledge PalHub relevan sama task (FTS5 di palhub-web).
    pub fn search(&self, query: &str, limit: usize) -> Vec<KnowledgeHit> {
        let url = format!(
            "{}/api/specialists/{}/knowledge/search?q={}&limit={}",
            self.base_url,
            self.specialist_id,
            urlencode(query),
            limit
        );
        match ureq::get(&url).timeout(std::time::Duration::from_secs(5)).call() {
            Ok(resp) => {
                let body = resp.into_string().unwrap_or_default();
                match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(v) => v
                        .get("items")
                        .and_then(|i| serde_json::from_value(i.clone()).ok())
                        .unwrap_or_default(),
                    Err(_) => vec![],
                }
            }
            Err(e) => {
                eprintln!("[palskills-engine] PalHub search failed (fail-open): {e}");
                vec![]
            }
        }
    }

    /// record_session: simpan session knowledge sebagai note PalHub. Return note id.
    pub fn record(&self, title: &str, content: &str, source: &str) -> Option<i64> {
        let url = format!("{}/api/specialists/{}/knowledge", self.base_url, self.specialist_id);
        let body = serde_json::json!({ "title": title, "content": content, "source": source });
        match ureq::post(&url).timeout(std::time::Duration::from_secs(5)).send_json(body) {
            Ok(resp) => {
                let resp_body = resp.into_string().unwrap_or_default();
                serde_json::from_str::<serde_json::Value>(&resp_body)
                    .ok()
                    .and_then(|v| v.get("id").and_then(|i| i.as_i64()))
            }
            Err(e) => {
                eprintln!("[palskills-engine] PalHub record failed (fail-open): {e}");
                None
            }
        }
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}