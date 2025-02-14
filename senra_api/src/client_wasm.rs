use wasm_bindgen::prelude::*;
use client::{Client, ApiError};
use serde::{Serialize, Deserialize};

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
    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, ApiError> {
        let request = Request::Login(LoginRequest { username, password });
        self.inner.request_with::<AuthResponse>(request).await.map(|auth| {
            self.inner.set_token(auth.token.clone());
            auth
        })
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, ApiError> {
        let request = Request::Register(RegisterRequest {
            username,
            email,
            password,
        });
        self.inner.request_with::<AuthResponse>(request).await.map(|auth| {
            self.set_token(auth.token.clone());
            auth
        })
    }

    #[wasm_bindgen]
    pub async fn verify_token(&mut self, token: String) -> Result<TokenResponse, ApiError> {
        let request = Request::Auth(AuthRequest { token });
        self.inner.request_with::<TokenResponse>(request).await.map(|token| {
            if let Some(token) = token.token.clone() {
                self.inner.set_token(token);
            }
            token
        })
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) {
        self.inner.clear_token();
    }

    #[wasm_bindgen]
    pub async fn get_self(&self) -> Result<UserResponse, ApiError> {
        let request = Request::GetSelf;
        self.inner.request_with::<UserResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn get_user(&self, id: u64) -> Result<UserResponse, ApiError> {
        let request = Request::GetUser(id);
        self.inner.request_with::<UserResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn update_user(&self, data: EditUserRequest) -> Result<UserResponse, ApiError> {
        let request = Request::EditUser(data);
        self.inner.request_with::<UserResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn list_notebooks(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
        category: Option<String>,
        search: Option<String>,
    ) -> Result<NotebookListResponse, ApiError> {
        let request = Request::GetNotebookList {
            page,
            limit,
            category,
            search,
        };
        self.inner.request_with::<NotebookListResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn get_notebook(&self, id: u64) -> Result<NotebookResponse, ApiError> {
        let request = Request::GetNotebook(id);
        self.inner.request_with::<NotebookResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn create_notebook(
        &self,
        data: CreateNotebookRequest,
    ) -> Result<NotebookResponse, ApiError> {
        let request = Request::CreateNotebook(data);
        self.inner.request_with::<NotebookResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn update_notebook(
        &self,
        id: u64,
        data: EditNotebookRequest,
    ) -> Result<NotebookResponse, ApiError> {
        let request = Request::EditNotebook(id, data);
        self.inner.request_with::<NotebookResponse>(request).await
    }

    #[wasm_bindgen]
    pub async fn delete_notebook(&self, id: u64) -> Result<(), ApiError> {
        let request = Request::RemoveNotebook(id);
        self.inner.request_with::<()>(request).await
    }
}
