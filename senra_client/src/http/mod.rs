mod client;

pub use client::*;

use senra_api::*;

use crate::Result;

macro_rules! api_method {
    ($name:ident, $method:expr, $endpoint:expr, $req_type:ty, $res_type:ty) => {
        pub async fn $name(&self, request: $req_type) -> Result<$res_type> {
            self.request($method, $endpoint, Some(request)).await
        }
    };
    ($name:ident, $method:expr, $endpoint:expr, $res_type:ty) => {
        pub async fn $name(&self) -> Result<$res_type> {
            self.request::<(), $res_type>($method, $endpoint, None)
                .await
        }
    };
}

impl HttpClient {
    api_method!(login, "POST", "/auth/login", LoginRequest, AuthResponse);
    api_method!(
        register,
        "POST",
        "/auth/register",
        RegisterRequest,
        AuthResponse
    );
    api_method!(
        verify_token,
        "POST",
        "/auth/verify",
        AuthRequest,
        TokenResponse
    );

    api_method!(get_self, "GET", "/user", UserResponse);
    api_method!(
        edit_user,
        "PATCH",
        "/user",
        EditUserRequest,
        UserInfoResponse
    );

    pub async fn get_user(&self, id: i64) -> Result<UserResponse> {
        self.request::<(), UserResponse>("GET", &format!("/user/{}", id), None)
            .await
    }

    pub async fn get_notebooks(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<NotebookListResponse> {
        let mut endpoint = "/notebooks".to_string();
        let mut params = Vec::new();

        if let Some(page) = page {
            params.push(format!("page={}", page));
        }
        if let Some(per_page) = per_page {
            params.push(format!("per_page={}", per_page));
        }

        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }

        self.request::<(), NotebookListResponse>("GET", &endpoint, None)
            .await
    }

    pub async fn get_notebook(&self, id: i64) -> Result<NotebookResponse> {
        self.request::<(), NotebookResponse>("GET", &format!("/notebooks/{}", id), None)
            .await
    }

    api_method!(
        create_notebook,
        "POST",
        "/notebooks",
        CreateNotebookRequest,
        NotebookResponse
    );

    pub async fn update_notebook(
        &self,
        id: i64,
        request: EditNotebookRequest,
    ) -> Result<NotebookResponse> {
        self.request("PATCH", &format!("/notebooks/{}", id), Some(request))
            .await
    }

    pub async fn delete_notebook(&self, id: i64) -> Result<()> {
        self.request::<(), ()>("DELETE", &format!("/notebooks/{}", id), None)
            .await
    }

    pub async fn like_notebook(&self, id: i64) -> Result<()> {
        self.request::<(), ()>("POST", &format!("/notebooks/{}/like", id), None)
            .await
    }

    pub async fn unlike_notebook(&self, id: i64) -> Result<()> {
        self.request::<(), ()>("POST", &format!("/notebooks/{}/unlike", id), None)
            .await
    }
}
