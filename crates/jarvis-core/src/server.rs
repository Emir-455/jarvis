use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde_json::json;
use tokio::sync::Notify;
use tracing::info;

use crate::router::CommandRouter;
use crate::state::SystemState;

struct AppState {
    system: Arc<SystemState>,
    router: Arc<CommandRouter>,
}

/// Health endpoint: GET /health
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let health = state.system.overall_health();
    let body = json!({
        "status": format!("{health:?}"),
        "mark": state.system.mark_version.0,
        "booted": state.system.is_booted(),
    });
    axum::Json(body)
}

/// WebSocket endpoint: GET /ws — accepts commands, returns LLM responses.
async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        use axum::extract::ws::Message;

        info!("WebSocket client connected");

        while let Some(Ok(msg)) = socket.recv().await {
            match msg {
                Message::Text(text) => {
                    let response = match state.router.handle(&text).await {
                        Ok(answer) => json!({ "type": "response", "text": answer }),
                        Err(e) => json!({ "type": "error", "message": e.to_string() }),
                    };
                    let payload: String = response.to_string();
                    if socket.send(Message::Text(payload)).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        info!("WebSocket client disconnected");
    })
}

/// Start the Axum HTTP + WS server.
pub async fn start_server(
    system: Arc<SystemState>,
    router: Arc<CommandRouter>,
    addr: SocketAddr,
    shutdown: Arc<Notify>,
) -> Result<()> {
    let state = Arc::new(AppState { system, router });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    info!(%addr, "HTTP/WS server starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown.notified().await;
        })
        .await?;

    Ok(())
}
