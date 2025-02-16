use js_sys::{Promise, Uint8Array};
use wasm_bindgen::prelude::*;

use super::*;

impl From<ApiError> for JsValue {
    fn from(error: ApiError) -> Self {
        js_sys::Error::new(&error.to_string()).into()
    }
}

#[wasm_bindgen]
pub struct JsUserInfoResponse {
    inner: UserInfoResponse,
}

#[wasm_bindgen]
impl JsUserInfoResponse {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u32 {
        self.inner.id as u32
    }

    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.inner.username.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        self.inner.email.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar(&self) -> Uint8Array {
        Uint8Array::from(self.inner.avatar.as_slice())
    }
}

#[wasm_bindgen]
pub struct JsClient {
    storage: Option<web_sys::Storage>,
    inner: Client,
}

#[wasm_bindgen]
impl JsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> Self {
        let storage = web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten();

        let mut client = Self {
            storage,
            inner: Client::new(base_url),
        };

        if let Some(storage) = &client.storage {
            if let Ok(Some(token)) = storage.get_item("token") {
                client.inner.set_token(token);
            }
        }

        client
    }

    #[wasm_bindgen(getter)]
    pub fn token(&self) -> Option<String> {
        self.inner.token.clone()
    }

    #[wasm_bindgen]
    pub fn login(&mut self, username: String, password: String) -> Promise {
        let mut client = self.inner.clone();
        let storage = self.storage.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let request = Request::Login(LoginRequest { username, password });
            let result = client.request_with::<AuthResponse>(request).await;
            match result {
                Ok(AuthResponse { token, user }) => {
                    if let Some(storage) = &storage {
                        let _ = storage.set_item("token", &token);
                    }
                    client.set_token(token);
                    let js_user = JsUserInfoResponse { inner: user };
                    Ok(JsValue::from(js_user))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn register(&mut self, username: String, email: String, password: String) -> Promise {
        let mut client = self.inner.clone();
        let storage = self.storage.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let request = Request::Register(RegisterRequest {
                username,
                email,
                password,
            });
            let result = client.request_with::<AuthResponse>(request).await;
            match result {
                Ok(AuthResponse { token, user }) => {
                    if let Some(storage) = &storage {
                        let _ = storage.set_item("token", &token);
                    }
                    client.set_token(token);
                    let js_user = JsUserInfoResponse { inner: user };
                    Ok(JsValue::from(js_user))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn verify_token(&mut self) -> Promise {
        let mut client = self.inner.clone();
        let storage = self.storage.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let token = storage
                .as_ref()
                .and_then(|s| s.get_item("token").ok())
                .flatten()
                .ok_or_else(|| JsValue::from(false))?;

            let request = Request::Auth(AuthRequest { token });
            let result = client.request_with::<TokenResponse>(request).await;
            match result {
                Ok(TokenResponse { token }) => {
                    if let Some(token) = token {
                        if let Some(storage) = &storage {
                            let _ = storage.set_item("token", &token);
                        }
                        client.set_token(token);
                    }
                    Ok(JsValue::from(true))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) {
        if let Some(storage) = &self.storage {
            let _ = storage.remove_item("token");
        }
        self.inner.clear_token();
    }
}
