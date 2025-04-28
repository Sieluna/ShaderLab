use serde_wasm_bindgen::{Serializer, from_value};
use wasm_bindgen::prelude::*;
use web_sys::window;

use crate::{ClientConfig, Error, HttpClient, WsClient};

use senra_api::*;

const JSON_SERIALIZER: Serializer = Serializer::json_compatible();

fn serialize_to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, ApiError> {
    value.serialize(&JSON_SERIALIZER).map_err(ApiError::from)
}

#[wasm_bindgen]
pub struct ApiError {
    message: String,
    code: String,
}

#[wasm_bindgen]
impl ApiError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let (message, code) = match err {
            Error::Authentication(msg) => (msg, "AUTH_ERROR".to_string()),
            Error::NotFound(msg) => (msg, "NOT_FOUND".to_string()),
            Error::BadRequest(msg) => (msg, "BAD_REQUEST".to_string()),
            Error::Network(msg) => (msg, "NETWORK_ERROR".to_string()),
            Error::InternalServerError(msg) => (msg, "SERVER_ERROR".to_string()),
            _ => (err.to_string(), "UNKNOWN_ERROR".to_string()),
        };
        Self { message, code }
    }
}

impl From<serde_wasm_bindgen::Error> for ApiError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self {
            message: err.to_string(),
            code: "SERIALIZATION_ERROR".to_string(),
        }
    }
}

#[wasm_bindgen]
pub struct ApiClient {
    client: HttpClient,
    ws_client: Option<WsClient>,
    token: Option<String>,
}

#[wasm_bindgen]
impl ApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> Result<ApiClient, ApiError> {
        console_error_panic_hook::set_once();

        let config = ClientConfig::new(base_url);
        let client = HttpClient::new(config).map_err(ApiError::from)?;

        Ok(ApiClient {
            client,
            ws_client: None,
            token: None,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token.clone();
        self.update_client_token();
    }

    fn update_client_token(&mut self) {
        if let Some(ref token_str) = self.token {
            let base_url = self.client.base_url().to_string();
            let mut config = ClientConfig::new(base_url);
            config = config.with_token(token_str.clone());
            if let Ok(new_client) = HttpClient::new(config) {
                self.client = new_client;
            }
        }
    }

    #[wasm_bindgen]
    pub fn save_token(&self) {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Some(ref token) = self.token {
                    let _ = storage.set_item("auth_token", token);
                } else {
                    let _ = storage.remove_item("auth_token");
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn load_token(&mut self) {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(token)) = storage.get_item("auth_token") {
                    self.set_token(Some(token));
                }
            }
        }
    }

    // Authentication API
    #[wasm_bindgen]
    pub async fn login(&mut self, username: &str, password: &str) -> Result<JsValue, ApiError> {
        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.login(request).await.map_err(ApiError::from)?;
        self.set_token(Some(response.token.clone()));
        self.save_token();
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<JsValue, ApiError> {
        let request = RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .register(request)
            .await
            .map_err(ApiError::from)?;
        self.set_token(Some(response.token.clone()));
        self.save_token();
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn verify_token(&self) -> Result<JsValue, ApiError> {
        if let Some(ref token) = self.token {
            let request = AuthRequest {
                token: token.clone(),
            };
            let response = self
                .client
                .verify_token(request)
                .await
                .map_err(ApiError::from)?;
            serialize_to_js(&response)
        } else {
            Err(ApiError {
                message: "No token available".to_string(),
                code: "NO_TOKEN".to_string(),
            })
        }
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) {
        self.set_token(None);
        self.save_token();
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    // User API
    #[wasm_bindgen]
    pub async fn get_self(&self) -> Result<JsValue, ApiError> {
        let response = self.client.get_self().await.map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn get_user(&self, id: u32) -> Result<JsValue, ApiError> {
        let response = self
            .client
            .get_user(id as i64)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn update_user(&self, data: JsValue) -> Result<JsValue, ApiError> {
        let request: EditUserRequest = from_value(data).map_err(ApiError::from)?;
        let response = self
            .client
            .edit_user(request)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    // Notebook API
    #[wasm_bindgen]
    pub async fn get_notebooks(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let page_i64 = page.map(|p| p as i64);
        let per_page_i64 = per_page.map(|p| p as i64);
        let response = self
            .client
            .get_notebooks(page_i64, per_page_i64)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn get_notebook(&self, id: u32) -> Result<JsValue, ApiError> {
        let response = self
            .client
            .get_notebook(id as i64)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn create_notebook(&self, data: JsValue) -> Result<JsValue, ApiError> {
        let request: CreateNotebookRequest = from_value(data).map_err(ApiError::from)?;
        let response = self
            .client
            .create_notebook(request)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn update_notebook(&self, id: u32, data: JsValue) -> Result<JsValue, ApiError> {
        let request: EditNotebookRequest = from_value(data).map_err(ApiError::from)?;
        let response = self
            .client
            .update_notebook(id as i64, request)
            .await
            .map_err(ApiError::from)?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn delete_notebook(&self, id: u32) -> Result<(), ApiError> {
        self.client
            .delete_notebook(id as i64)
            .await
            .map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn like_notebook(&self, id: u32) -> Result<(), ApiError> {
        self.client
            .like_notebook(id as i64)
            .await
            .map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn unlike_notebook(&self, id: u32) -> Result<(), ApiError> {
        self.client
            .unlike_notebook(id as i64)
            .await
            .map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn connect_websocket(&mut self) -> Result<(), ApiError> {
        if self.ws_client.is_none() {
            let base_url = self.client.base_url().to_string();
            let mut config = ClientConfig::new(base_url);
            if let Some(ref token) = self.token {
                config = config.with_token(token.clone());
            }
            let ws_client = WsClient::new(config).await.map_err(ApiError::from)?;
            ws_client.connect().await.map_err(ApiError::from)?;
            self.ws_client = Some(ws_client);
        } else if let Some(ref ws_client) = self.ws_client {
            // Reconnect if not connected
            if !ws_client.is_connected().await {
                ws_client.connect().await.map_err(ApiError::from)?;
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn disconnect_websocket(&mut self) -> Result<(), ApiError> {
        if let Some(ws_client) = self.ws_client.take() {
            ws_client.close().await.map_err(ApiError::from)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn is_websocket_connected(&self) -> bool {
        if let Some(ref ws_client) = self.ws_client {
            ws_client.is_connected().await
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub async fn send_ws_message(
        &self,
        message_type: &str,
        payload: JsValue,
    ) -> Result<(), ApiError> {
        if let Some(ref ws_client) = self.ws_client {
            let payload_value: serde_json::Value = from_value(payload).map_err(ApiError::from)?;
            ws_client
                .send_message(message_type, payload_value)
                .await
                .map_err(ApiError::from)
        } else {
            Err(ApiError {
                message: "WebSocket not connected".to_string(),
                code: "WS_NOT_CONNECTED".to_string(),
            })
        }
    }

    #[wasm_bindgen]
    pub async fn receive_ws_message(&self) -> Result<JsValue, ApiError> {
        if let Some(ref ws_client) = self.ws_client {
            let response: crate::ws::WsResponse<serde_json::Value> =
                ws_client.receive_response().await.map_err(ApiError::from)?;
            serialize_to_js(&response)
        } else {
            Err(ApiError {
                message: "WebSocket not connected".to_string(),
                code: "WS_NOT_CONNECTED".to_string(),
            })
        }
    }
}
