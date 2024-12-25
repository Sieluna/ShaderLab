use axum::Router;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use serde::Deserialize;
use tracing::{debug, info};

use crate::errors::Result;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct WsQuery {
    token: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let user_id = state.services.auth.authorize(&query.token).await?;
    info!("WebSocket connection established for user {}", user_id);

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket, state: AppState, user_id: i64) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            debug!("Received WebSocket message: {}", text);
        }
    }

    info!("WebSocket connection closed for user {}", user_id);
}
