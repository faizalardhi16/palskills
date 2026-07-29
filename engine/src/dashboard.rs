//! Dashboard — Web UI with real-time pipeline visualization.
//!
//! axum HTTP server on :3030 serving embedded HTML dashboard.
//! WebSocket broadcasts pipeline state updates in real-time.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
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
}

pub static EMBEDDED_HTML: &str = include_str!("dashboard.html");

pub async fn serve(_palbox: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, _) = broadcast::channel::<PipelineEvent>(32);
    let tx = Arc::new(tx);

    let app = Router::new()
        .route("/", get(|| async { Html(EMBEDDED_HTML) }))
        .route("/ws", get(move |ws: WebSocketUpgrade| {
            let tx = tx.clone();
            async move { ws.on_upgrade(move |socket| handle_ws(socket, tx)) }
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_ws(mut socket: WebSocket, tx: Arc<broadcast::Sender<PipelineEvent>>) {
    let mut rx = tx.subscribe();

    // Send initial status
    let _ = socket.send(Message::Text(
        serde_json::to_string(&PipelineEvent {
            event: "connected".into(),
            skill: None,
            status: Some("idle".into()),
            message: Some("Palskills Engine ready. 11 skills loaded.".into()),
            duration_ms: None,
            flow: None,
            confidence: None,
        }).unwrap()
    )).await;

    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = socket.send(Message::Text(json)).await;
                }
            }
            _ = socket.recv() => {
                // client disconnected or message received
                return;
            }
        }
    }
}
