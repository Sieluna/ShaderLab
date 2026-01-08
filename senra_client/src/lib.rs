pub mod client;
mod config;
pub mod ws;

pub use client::ApiClient;
pub use config::ClientConfig;
pub use ws::{Message, WebSocket, WsClient};

use senra_api::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
const JSON_SERIALIZER: serde_wasm_bindgen::Serializer =
    serde_wasm_bindgen::Serializer::json_compatible();

#[cfg(target_arch = "wasm32")]
fn serialize_to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, ApiError> {
    Ok(value.serialize(&JSON_SERIALIZER)?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ApiError {
    message: String,
    code: String,
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
impl From<serde_wasm_bindgen::Error> for ApiError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        crate::client::Error::from(err).into()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<crate::client::Error> for ApiError {
    fn from(err: crate::client::Error) -> Self {
        let (message, code) = match err {
            crate::client::Error::Http(e) => {
                if let Some(status) = e.status() {
                    (
                        format!("HTTP {}: {}", status.as_u16(), e),
                        "HTTP_ERROR".to_string(),
                    )
                } else {
                    (format!("Network error: {}", e), "NETWORK_ERROR".to_string())
                }
            }
            crate::client::Error::WebSocket(e) => (
                format!("WebSocket error: {}", e),
                "WEBSOCKET_ERROR".to_string(),
            ),
            crate::client::Error::UrlParse(e) => (
                format!("URL parse error: {}", e),
                "URL_PARSE_ERROR".to_string(),
            ),
            crate::client::Error::Auth(msg) => (msg, "AUTH_ERROR".to_string()),
            crate::client::Error::Serialization(msg) => (msg, "SERIALIZATION_ERROR".to_string()),
        };
        Self { message, code }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<crate::ws::Error> for ApiError {
    fn from(err: crate::ws::Error) -> Self {
        crate::client::Error::WebSocket(err).into()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        crate::client::Error::Serialization(err.to_string()).into()
    }
}

#[cfg(target_arch = "wasm32")]
fn save_token_to_storage(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn clear_token_from_storage() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
        }
    }
}

impl ApiClient {
    // ========== Authentication API ==========

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> crate::client::Result<AuthResponse> {
        let url = self.http_url().join("/auth/login")?;
        let body = LoginRequest { username, password };
        let req = self.http().post(url).json(&body);

        let response = req.send().await?;

        let result = response.json::<AuthResponse>().await?;
        self.set_token(result.token.clone());
        Ok(result)
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> crate::client::Result<AuthResponse> {
        let url = self.http_url().join("/auth/register")?;
        let body = RegisterRequest {
            username,
            email,
            password,
        };
        let req = self.http().post(url).json(&body);

        let response = req.send().await?;

        let result = response.json::<AuthResponse>().await?;
        self.set_token(result.token.clone());
        Ok(result)
    }

    pub async fn verify_token(&mut self) -> crate::client::Result<TokenResponse> {
        let url = self.http_url().join("/auth/verify")?;
        let body = AuthRequest {
            token: self
                .token()
                .ok_or(crate::client::Error::Auth("No token available".to_string()))?
                .to_string(),
        };
        let req = self.http().post(url).json(&body);

        let response = req.send().await?;

        let result = response.json::<TokenResponse>().await?;
        if let Some(new_token) = result.token.clone() {
            self.set_token(new_token);
        }
        Ok(result)
    }

    pub fn logout(&mut self) {
        self.clear_token();
    }

    // ========== User API ==========

    pub async fn get_self(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::client::Result<UserResponse> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);

        let url = self
            .http_url()
            .join(&format!("/user?page={page}&per_page={per_page}"))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<UserResponse>().await?;
        Ok(result)
    }

    pub async fn get_user(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::client::Result<UserResponse> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);

        let url = self
            .http_url()
            .join(&format!("/user/{id}?page={page}&per_page={per_page}"))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<UserResponse>().await?;
        Ok(result)
    }

    pub async fn update_user(
        &mut self,
        request: EditUserRequest,
    ) -> crate::client::Result<UserInfoResponse> {
        let url = self.http_url().join("/user")?;
        let mut req = self.http().patch(url).json(&request);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<UserInfoResponse>().await?;
        Ok(result)
    }

    // ========== Notebook API ==========

    pub async fn get_notebooks(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::client::Result<NotebookListResponse> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);

        let url = self
            .http_url()
            .join(&format!("/notebooks?page={page}&per_page={per_page}"))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookListResponse>().await?;
        Ok(result)
    }

    pub async fn get_notebook(&self, id: u32) -> crate::client::Result<NotebookResponse> {
        let url = self.http_url().join(&format!("/notebooks/{id}"))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookResponse>().await?;
        Ok(result)
    }

    pub async fn create_notebook(
        &mut self,
        request: CreateNotebookRequest,
    ) -> crate::client::Result<NotebookResponse> {
        let url = self.http_url().join("/notebooks")?;
        let mut req = self.http().post(url).json(&request);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookResponse>().await?;
        Ok(result)
    }

    pub async fn update_notebook(
        &mut self,
        id: u32,
        request: EditNotebookRequest,
    ) -> crate::client::Result<NotebookResponse> {
        let url = self.http_url().join(&format!("/notebooks/{id}"))?;
        let mut req = self.http().patch(url).json(&request);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookResponse>().await?;
        Ok(result)
    }

    pub async fn delete_notebook(&mut self, id: u32) -> crate::client::Result<()> {
        let url = self.http_url().join(&format!("/notebooks/{id}"))?;
        let mut req = self.http().delete(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        req.send().await?;

        Ok(())
    }

    pub async fn like_notebook(&mut self, id: u32) -> crate::client::Result<()> {
        let url = self.http_url().join(&format!("/notebooks/{id}/like"))?;
        let mut req = self.http().post(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        req.send().await?;

        Ok(())
    }

    pub async fn unlike_notebook(&mut self, id: u32) -> crate::client::Result<()> {
        let url = self.http_url().join(&format!("/notebooks/{id}/unlike"))?;
        let mut req = self.http().post(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        req.send().await?;

        Ok(())
    }

    pub async fn get_notebook_versions(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::client::Result<NotebookVersionListResponse> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);

        let url = self.http_url().join(&format!(
            "/notebooks/{id}/versions?page={page}&per_page={per_page}"
        ))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookVersionListResponse>().await?;
        Ok(result)
    }

    pub async fn get_notebook_comments(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::client::Result<NotebookCommentListResponse> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);

        let url = self.http_url().join(&format!(
            "/notebooks/{id}/comments?page={page}&per_page={per_page}"
        ))?;
        let mut req = self.http().get(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookCommentListResponse>().await?;
        Ok(result)
    }

    pub async fn create_notebook_comment(
        &mut self,
        id: u32,
        request: CreateNotebookCommentRequest,
    ) -> crate::client::Result<NotebookCommentResponse> {
        let url = self.http_url().join(&format!("/notebooks/{id}/comments"))?;
        let mut req = self.http().post(url).json(&request);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        let result = response.json::<NotebookCommentResponse>().await?;
        Ok(result)
    }

    pub async fn delete_notebook_comment(
        &mut self,
        id: u32,
        comment_id: u32,
    ) -> crate::client::Result<()> {
        let url = self
            .http_url()
            .join(&format!("/notebooks/{id}/comments/{comment_id}"))?;
        let mut req = self.http().delete(url);
        if let Some(token) = self.token() {
            req = req.bearer_auth(token);
        }

        req.send().await?;

        Ok(())
    }
}

// WASM implementation - constructor
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ApiClient {
    #[wasm_bindgen(constructor)]
    pub fn wasm_constructor(base_url: String) -> Result<ApiClient, ApiError> {
        let url = url::Url::parse(&base_url).map_err(|e| ApiError {
            message: format!("Invalid URL: {}", e),
            code: "URL_PARSE_ERROR".to_string(),
        })?;

        let config = crate::config::ClientConfig::new(url, false);
        Ok(crate::client::ApiClient::new(config)?)
    }

    // ========== Authentication API ==========

    #[wasm_bindgen]
    pub async fn wasm_login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<JsValue, ApiError> {
        let response = Self::login(self, username, password).await?;
        save_token_to_storage(&response.token);
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, ApiError> {
        let response = Self::register(self, username, email, password).await?;
        save_token_to_storage(&response.token);
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_verify_token(&mut self) -> Result<JsValue, ApiError> {
        let response = Self::verify_token(self).await?;
        if let Some(ref new_token) = response.token {
            save_token_to_storage(new_token);
        }
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub fn wasm_logout(&mut self) {
        Self::logout(self);
        clear_token_from_storage();
    }

    #[wasm_bindgen]
    pub fn wasm_load_token(&mut self) {
        if let Some(w) = web_sys::window() {
            if let Ok(Some(storage)) = w.local_storage() {
                if let Ok(Some(token)) = storage.get_item("auth_token") {
                    if !token.is_empty() {
                        self.set_token(token);
                    }
                }
            }
        }
    }

    // ========== User API ==========

    #[wasm_bindgen]
    pub async fn wasm_get_self(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let response = Self::get_self(self, page, per_page).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_get_user(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let response = Self::get_user(self, id, page, per_page).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_update_user(&mut self, data: JsValue) -> Result<JsValue, ApiError> {
        let request: EditUserRequest = serde_wasm_bindgen::from_value(data)?;
        let response = Self::update_user(self, request).await?;
        serialize_to_js(&response)
    }

    // ========== Notebook API ==========

    #[wasm_bindgen]
    pub async fn wasm_get_notebooks(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let response = Self::get_notebooks(self, page, per_page).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_get_notebook(&self, id: u32) -> Result<JsValue, ApiError> {
        let response = Self::get_notebook(self, id).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_create_notebook(&mut self, data: JsValue) -> Result<JsValue, ApiError> {
        let request: CreateNotebookRequest = serde_wasm_bindgen::from_value(data)?;
        let response = Self::create_notebook(self, request).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_update_notebook(
        &mut self,
        id: u32,
        data: JsValue,
    ) -> Result<JsValue, ApiError> {
        let request: EditNotebookRequest = serde_wasm_bindgen::from_value(data)?;
        let response = Self::update_notebook(self, id, request).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_delete_notebook(&mut self, id: u32) -> Result<(), ApiError> {
        Self::delete_notebook(self, id)
            .await
            .map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn wasm_like_notebook(&mut self, id: u32) -> Result<(), ApiError> {
        Self::like_notebook(self, id).await.map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn wasm_unlike_notebook(&mut self, id: u32) -> Result<(), ApiError> {
        Self::unlike_notebook(self, id)
            .await
            .map_err(ApiError::from)
    }

    #[wasm_bindgen]
    pub async fn wasm_get_notebook_versions(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let response = Self::get_notebook_versions(self, id, page, per_page).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_get_notebook_comments(
        &self,
        id: u32,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<JsValue, ApiError> {
        let response = Self::get_notebook_comments(self, id, page, per_page).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_create_notebook_comment(
        &mut self,
        id: u32,
        data: JsValue,
    ) -> Result<JsValue, ApiError> {
        let request: CreateNotebookCommentRequest = serde_wasm_bindgen::from_value(data)?;
        let response = Self::create_notebook_comment(self, id, request).await?;
        serialize_to_js(&response)
    }

    #[wasm_bindgen]
    pub async fn wasm_delete_notebook_comment(
        &mut self,
        id: u32,
        comment_id: u32,
    ) -> Result<(), ApiError> {
        Self::delete_notebook_comment(self, id, comment_id)
            .await
            .map_err(ApiError::from)
    }

    // ========== WebSocket API ==========

    #[wasm_bindgen]
    pub async fn wasm_connect_websocket(&mut self) -> Result<(), ApiError> {
        let mut ws_url = self.ws_url().clone();

        // Add token to WebSocket URL if available
        if let Some(token) = self.token() {
            ws_url.query_pairs_mut().append_pair("token", token);
        }

        self.ws_mut().connect(ws_url).await.map_err(|e| ApiError {
            message: format!("WebSocket connection failed: {}", e),
            code: "WEBSOCKET_ERROR".to_string(),
        })?;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub async fn wasm_send_ws_message(
        &mut self,
        message_type: String,
        payload: JsValue,
    ) -> Result<(), ApiError> {
        let payload_value: serde_json::Value = serde_wasm_bindgen::from_value(payload)?;

        // Build message according to server's WsMessage format
        let id = format!("{:x}", js_sys::Date::now() as u64);
        let message = serde_json::json!({
            "id": id,
            "message_type": message_type,
            "payload": payload_value,
            "timestamp": js_sys::Date::now() as u64,
        });

        // Use JSON codec for encoding
        let json_codec = ProtocolCodec::Json(JsonCodec);
        self.ws_mut().send(&message, &json_codec).await?;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub async fn wasm_receive_ws_message(&mut self) -> Result<JsValue, ApiError> {
        // Try to decode using JSON codec first
        let json_codec = ProtocolCodec::Json(JsonCodec);
        if let Some(value) = self
            .ws_mut()
            .receive::<serde_json::Value>(&json_codec)
            .await
            .map_err(ApiError::from)?
        {
            return serialize_to_js(&value).map_err(ApiError::from);
        }

        // Fallback to raw message
        let message = self
            .ws_mut()
            .receive_message()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| crate::client::Error::WebSocket(crate::ws::Error::NotConnected))?;

        match message {
            crate::ws::Message::Text(text) => {
                let value: serde_json::Value = serde_json::from_str(&text)?;
                serialize_to_js(&value)
            }
            crate::ws::Message::Binary(data) => {
                // For binary messages, return as base64 string
                let base64 = js_sys::Uint8Array::from(&data[..]);
                Ok(JsValue::from_str(&format!("binary:{}", base64.to_string())))
            }
        }
    }
}
