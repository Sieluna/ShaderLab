use async_trait::async_trait;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use senra_api::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::errors::Result;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
struct WsQuery {
    token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub id: String,
    pub message_type: String,
    pub payload: T,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse<T> {
    pub request_id: String,
    pub success: bool,
    pub payload: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsNotification<T> {
    pub notification_type: String,
    pub payload: T,
    pub timestamp: u64,
}

#[async_trait]
trait MessageHandler {
    async fn handle(
        &self,
        state: &AppState,
        user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value>;
}

struct PingHandler;

#[async_trait]
impl MessageHandler for PingHandler {
    async fn handle(
        &self,
        _state: &AppState,
        _user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value> {
        WsResponse {
            request_id: message.id.clone(),
            success: true,
            payload: Some(Value::String("pong".to_string())),
            error: None,
        }
    }
}

struct NotebookSubscribeHandler;

#[async_trait]
impl MessageHandler for NotebookSubscribeHandler {
    async fn handle(
        &self,
        _state: &AppState,
        user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value> {
        if let Some(notebook_id) = message.payload.get("notebook_id").and_then(|v| v.as_i64()) {
            info!("User {} subscribed to notebook {}", user_id, notebook_id);

            // TODO:
            // 1. Verify user has access to the notebook
            // 2. Add user to a subscription list for the notebook
            // 3. Set up real-time updates for the notebook

            WsResponse {
                request_id: message.id.clone(),
                success: true,
                payload: Some(Value::String(format!(
                    "Subscribed to notebook {}",
                    notebook_id
                ))),
                error: None,
            }
        } else {
            WsResponse {
                request_id: message.id.clone(),
                success: false,
                payload: None,
                error: Some("Missing or invalid notebook_id".to_string()),
            }
        }
    }
}

struct NotebookUnsubscribeHandler;

#[async_trait]
impl MessageHandler for NotebookUnsubscribeHandler {
    async fn handle(
        &self,
        _state: &AppState,
        user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value> {
        if let Some(notebook_id) = message.payload.get("notebook_id").and_then(|v| v.as_i64()) {
            info!(
                "User {} unsubscribed from notebook {}",
                user_id, notebook_id
            );

            // TODO:
            // 1. Remove user from the subscription list for the notebook

            WsResponse {
                request_id: message.id.clone(),
                success: true,
                payload: Some(Value::String(format!(
                    "Unsubscribed from notebook {}",
                    notebook_id
                ))),
                error: None,
            }
        } else {
            WsResponse {
                request_id: message.id.clone(),
                success: false,
                payload: None,
                error: Some("Missing or invalid notebook_id".to_string()),
            }
        }
    }
}

struct RealtimeEditHandler;

#[async_trait]
impl MessageHandler for RealtimeEditHandler {
    async fn handle(
        &self,
        _state: &AppState,
        user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value> {
        debug!(
            "Received realtime edit from user {}: {:?}",
            user_id, message.payload
        );

        // TODO:
        // 1. Parse the edit operation (insert, delete, format, etc.)
        // 2. Apply operational transformation to resolve conflicts
        // 3. Broadcast the transformed operation to other connected users
        // 4. Optionally save a snapshot of the current state

        WsResponse {
            request_id: message.id.clone(),
            success: true,
            payload: Some(Value::String("Edit processed".to_string())),
            error: None,
        }
    }
}

struct MessageRouter {
    handlers: std::collections::HashMap<String, Box<dyn MessageHandler + Send + Sync>>,
}

impl MessageRouter {
    fn new() -> Self {
        let mut handlers: std::collections::HashMap<String, Box<dyn MessageHandler + Send + Sync>> =
            std::collections::HashMap::new();

        handlers.insert("ping".to_string(), Box::new(PingHandler));
        handlers.insert(
            "notebook_subscribe".to_string(),
            Box::new(NotebookSubscribeHandler),
        );
        handlers.insert(
            "notebook_unsubscribe".to_string(),
            Box::new(NotebookUnsubscribeHandler),
        );
        handlers.insert("realtime_edit".to_string(), Box::new(RealtimeEditHandler));

        Self { handlers }
    }

    async fn route(
        &self,
        state: &AppState,
        user_id: i64,
        message: &WsMessage<Value>,
    ) -> WsResponse<Value> {
        if let Some(handler) = self.handlers.get(&message.message_type) {
            handler.handle(state, user_id, message).await
        } else {
            warn!("Unknown message type: {}", message.message_type);
            WsResponse {
                request_id: message.id.clone(),
                success: false,
                payload: None,
                error: Some("Unknown message type".to_string()),
            }
        }
    }
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
    info!("WebSocket connection established for user {}", user_id);

    let router = MessageRouter::new();
    let protocol_codec = ProtocolCodec::default();

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(axum::extract::ws::Message::Binary(data)) => {
                if let Err(e) = handle_message(
                    &mut socket,
                    &state,
                    &router,
                    &protocol_codec,
                    user_id,
                    data,
                    false,
                )
                .await
                {
                    error!("Error handling binary message: {}", e);
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Text(text)) => {
                if let Err(e) = handle_message(
                    &mut socket,
                    &state,
                    &router,
                    &protocol_codec,
                    user_id,
                    text.into(),
                    true,
                )
                .await
                {
                    error!("Error handling text message: {}", e);
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                info!("WebSocket connection closed normally for user {}", user_id);
                break;
            }
            Ok(axum::extract::ws::Message::Ping(data)) => {
                if socket
                    .send(axum::extract::ws::Message::Pong(data))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Pong(_)) => {
                // Handle pong if needed
            }
            Err(e) => {
                warn!("WebSocket error for user {}: {}", user_id, e);
                break;
            }
        }
    }

    info!("WebSocket connection closed for user {}", user_id);
}

async fn handle_message(
    socket: &mut axum::extract::ws::WebSocket,
    state: &AppState,
    router: &MessageRouter,
    protocol_codec: &ProtocolCodec,
    user_id: i64,
    data: Bytes,
    is_text: bool,
) -> Result<()> {
    let message: WsMessage<Value> = if is_text {
        serde_json::from_slice(&data).map_err(|e| {
            crate::errors::AppError::ValidationError(format!("Failed to parse JSON: {}", e))
        })?
    } else {
        protocol_codec.decode(&data).map_err(|e| {
            crate::errors::AppError::ValidationError(format!("Failed to decode message: {}", e))
        })?
    };

    debug!(
        "Received WebSocket message type: {} from user {}",
        message.message_type, user_id
    );

    let response = router.route(state, user_id, &message).await;

    send_response(socket, &response, protocol_codec, is_text).await
}

async fn send_response<T: Serialize>(
    socket: &mut axum::extract::ws::WebSocket,
    response: &T,
    protocol_codec: &ProtocolCodec,
    as_text: bool,
) -> Result<()> {
    if as_text {
        let response_text = serde_json::to_string(response).map_err(|e| {
            crate::errors::AppError::ValidationError(format!("Failed to serialize response: {}", e))
        })?;

        socket
            .send(axum::extract::ws::Message::Text(response_text.into()))
            .await
            .map_err(|e| {
                crate::errors::AppError::ValidationError(format!("Failed to send response: {}", e))
            })?;
    } else {
        let response_data = protocol_codec.encode(response).map_err(|e| {
            crate::errors::AppError::ValidationError(format!("Failed to encode response: {}", e))
        })?;

        socket
            .send(axum::extract::ws::Message::Binary(Bytes::from(
                response_data,
            )))
            .await
            .map_err(|e| {
                crate::errors::AppError::ValidationError(format!("Failed to send response: {}", e))
            })?;
    }

    Ok(())
}
