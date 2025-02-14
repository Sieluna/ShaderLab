use reqwest::{Client as HttpClient, header};
use serde::de::DeserializeOwned;

use super::*;

#[derive(Clone)]
pub struct Client {
    pub base_url: String,
    pub http_client: HttpClient,
    pub token: Option<String>,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let http_client = HttpClient::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: base_url.into(),
            http_client,
            token: None,
        }
    }

    pub fn url(&self) -> &str {
        &self.base_url
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub async fn request(&self, request: Request) -> Result<Response, ApiError> {
        Ok(match &request {
            Request::Auth(_) => {
                self.request_with::<TokenResponse>(request).await.map(|token| Response::Token(token))?
            },
            Request::Login(_) | Request::Register(_) => {
                self.request_with::<AuthResponse>(request).await.map(|auth| Response::Auth(auth))?
            },
            Request::GetSelf | Request::GetUser(_) | Request::EditUser(_) => {
                self.request_with::<UserResponse>(request).await.map(|user| Response::User(user))?
            }
            Request::GetNotebookList { .. } => {
                self.request_with::<NotebookListResponse>(request).await.map(|notebook_list| Response::NotebookList(notebook_list))?
            }
            Request::GetNotebook(_)
            | Request::CreateNotebook(_)
            | Request::EditNotebook(_, _)
            | Request::RemoveNotebook(_)
            | Request::UpdateShader { .. }
            | Request::UpdateResource { .. }
            | Request::LikeNotebook(_)
            | Request::UnlikeNotebook(_) => {
                self.request_with::<NotebookResponse>(request).await.map(|notebook| Response::Notebook(notebook))?
            }
            Request::GetCommentList { .. } => {
                self.request_with::<NotebookCommentListResponse>(request).await.map(|comment_list| Response::CommentList(comment_list))?
            }
            Request::CreateComment(_, _) => {
                self.request_with::<NotebookCommentResponse>(request).await.map(|comment| Response::Comment(comment))?
            },
        })
    }

    pub async fn request_with<T: DeserializeOwned>(&self, request: Request) -> Result<T, ApiError> {
        let endpoint: Endpoint = request.try_into()?;
        let url = format!("{}{}", self.base_url, endpoint.path);

        let request_builder = match endpoint.method {
            http::Method::GET => self.http_client.get(&url),
            http::Method::POST => self.http_client.post(&url),
            http::Method::PUT => self.http_client.put(&url),
            http::Method::DELETE => self.http_client.delete(&url),
            http::Method::PATCH => self.http_client.patch(&url),
            _ => {
                return Err(ApiError::UnknownError(format!(
                    "Unsupported method: {:?}",
                    endpoint.method
                )));
            }
        };

        let request_builder = endpoint
            .params
            .iter()
            .fold(request_builder, |builder, (key, value)| {
                builder.query(&[(key, value)])
            });

        let request_builder = if let Some(body) = endpoint.body {
            request_builder.json(&body)
        } else {
            request_builder
        };

        let request_builder = if let Some(token) = &self.token {
            request_builder.header(header::AUTHORIZATION, format!("Bearer {}", token))
        } else {
            request_builder
        };

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(ApiError::HttpError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let json = response.json::<serde_json::Value>().await?;

        Ok(serde_json::from_value(json)?)
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, ApiError> {
        let request = Request::Login(LoginRequest { username, password });
        self.request_with::<AuthResponse>(request).await.map(|auth| {
            self.set_token(auth.token.clone());
            auth
        })
    }

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
        self.request_with::<AuthResponse>(request).await.map(|auth| {
            self.set_token(auth.token.clone());
            auth
        })
    }

    pub async fn verify_token(&mut self, token: String) -> Result<TokenResponse, ApiError> {
        let request = Request::Auth(AuthRequest { token });
        self.request_with::<TokenResponse>(request).await.map(|token| {
            if let Some(token) = token.token.clone() {
                self.set_token(token);
            }
            token
        })
    }

    pub fn logout(&mut self) {
        self.clear_token();
    }

    pub async fn get_self(&self) -> Result<UserResponse, ApiError> {
        let request = Request::GetSelf;
        self.request_with::<UserResponse>(request).await
    }

    pub async fn get_user(&self, id: u64) -> Result<UserResponse, ApiError> {
        let request = Request::GetUser(id);
        self.request_with::<UserResponse>(request).await
    }

    pub async fn update_user(&self, data: EditUserRequest) -> Result<UserResponse, ApiError> {
        let request = Request::EditUser(data);
        self.request_with::<UserResponse>(request).await
    }

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
        self.request_with::<NotebookListResponse>(request).await
    }

    pub async fn get_notebook(&self, id: u64) -> Result<NotebookResponse, ApiError> {
        let request = Request::GetNotebook(id);
        self.request_with::<NotebookResponse>(request).await
    }

    pub async fn create_notebook(
        &self,
        data: CreateNotebookRequest,
    ) -> Result<NotebookResponse, ApiError> {
        let request = Request::CreateNotebook(data);
        self.request_with::<NotebookResponse>(request).await
    }

    pub async fn update_notebook(
        &self,
        id: u64,
        data: EditNotebookRequest,
    ) -> Result<NotebookResponse, ApiError> {
        let request = Request::EditNotebook(id, data);
        self.request_with::<NotebookResponse>(request).await
    }

    pub async fn delete_notebook(&self, id: u64) -> Result<(), ApiError> {
        let request = Request::RemoveNotebook(id);
        self.request_with::<()>(request).await
    }
}
