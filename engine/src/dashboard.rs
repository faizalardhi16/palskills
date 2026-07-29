//! Dashboard — Read-only Web UI showing pipeline state from .palbox/State.md.
//!
//! State.md is the single source of truth — written by MCP tools, displayed here.
//! WebSocket broadcasts pipeline events; on connect, sends current State.md snapshot.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::broadcast;

/// Pipeline event broadcast to all WebSocket clients
#[derive(Clone, Debug, serde::Serialize)]
pub struct PipelineEvent {
    pub event: String,
    pub skill: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub duration_ms: Option<u64>,
    pub flow: Option<Vec<String>>,
    pub confidence: Option<u8>,
    pub stats_nodes: Option<usize>,
    pub stats_symbols: Option<usize>,
    pub stats_files: Option<usize>,
    /// Full state snapshot (sent on connect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<crate::generator::PipelineState>,
}

/// Global broadcast sender for dashboard events.
pub static DASHBOARD_TX: OnceLock<Arc<broadcast::Sender<PipelineEvent>>> = OnceLock::new();

/// Send an event to all dashboard WebSocket clients. No-op if dashboard not started.
pub fn emit_event(event: PipelineEvent) {
    if let Some(tx) = DASHBOARD_TX.get() {
        let _ = tx.send(event);
    }
}

pub static EMBEDDED_HTML: &str = include_str!("dashboard.html");

pub async fn serve(palbox: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, _) = broadcast::channel::<PipelineEvent>(64);

    let tx = Arc::new(tx);
    DASHBOARD_TX
        .set(tx.clone())
        .map_err(|_| "DASHBOARD_TX already initialized")?;

    let tx_ws = tx.clone();

    let app = Router::new()
        .route("/", get(|| async { Html(EMBEDDED_HTML) }))
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| {
                let tx = tx_ws.clone();
                let palbox = palbox.clone();
                async move {
                    ws.on_upgrade(move |socket| handle_ws(socket, tx, palbox))
                }
            }),
        );

    log::info!("🌐 Dashboard listening on http://0.0.0.0:3030");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_ws(
    mut socket: WebSocket,
    tx: Arc<broadcast::Sender<PipelineEvent>>,
    palbox: PathBuf,
) {
    let mut rx = tx.subscribe();

    // Send full State.md snapshot on connect
    let state = crate::generator::read_state(&palbox);
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&PipelineEvent {
                event: "state_snapshot".into(),
                skill: None,
                status: None,
                message: None,
                duration_ms: None,
                flow: state.flow.clone(),
                confidence: state.confidence,
                stats_nodes: state.stats_nodes,
                stats_symbols: state.stats_symbols,
                stats_files: state.stats_files,
                state: Some(state),
            })
            .unwrap(),
        ))
        .await;

    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&event) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        return;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => return,
                    _ => continue, // ignore all incoming messages
                }
            }
        }
    }
}
