use wasm_bindgen::prelude::*;
use js_sys::Promise;

use super::*;

impl From<ApiError> for JsValue {
    fn from(error: ApiError) -> Self {
        js_sys::Error::new(&error.to_string()).into()
    }
}

#[wasm_bindgen]
pub struct JsAuthResponse {
    inner: AuthResponse,
}

#[wasm_bindgen]
impl JsAuthResponse {
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> String {
        self.inner.token.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> i64 {
        self.inner.user.id
    }
    
    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.inner.user.username.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        self.inner.user.email.clone()
    }
}

#[wasm_bindgen]
pub struct JsTokenResponse {
    inner: TokenResponse,
}

#[wasm_bindgen]
impl JsTokenResponse {
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> Option<String> {
        self.inner.token.clone()
    }
}

#[wasm_bindgen]
pub struct JsClient {
    inner: Client,
}

#[wasm_bindgen]
impl JsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> Self {
        Self {
            inner: Client::new(base_url)
        }
    }

    #[wasm_bindgen(getter)]
    pub fn token(&self) -> Option<String> {
        self.inner.token.clone()
    }

    #[wasm_bindgen]
    pub fn login(&mut self, username: String, password: String) -> Promise {
        let client = self.inner.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let request = Request::Login(LoginRequest { username, password });
            let result = client.request_with::<AuthResponse>(request).await;
            match result {
                Ok(auth) => {
                    let js_auth = JsAuthResponse { inner: auth };
                    Ok(JsValue::from(js_auth))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn register(&mut self, username: String, email: String, password: String) -> Promise {
        let client = self.inner.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let request = Request::Register(RegisterRequest {
                username,
                email,
                password,
            });
            let result = client.request_with::<AuthResponse>(request).await;
            match result {
                Ok(auth) => {
                    let js_auth = JsAuthResponse { inner: auth };
                    Ok(JsValue::from(js_auth))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn verify_token(&mut self, token: String) -> Promise {
        let client = self.inner.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let request = Request::Auth(AuthRequest { token });
            let result = client.request_with::<TokenResponse>(request).await;
            match result {
                Ok(token_resp) => {
                    let js_token = JsTokenResponse { inner: token_resp };
                    Ok(JsValue::from(js_token))
                }
                Err(err) => Err(err.into()),
            }
        })
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) {
        self.inner.clear_token();
    }
}
