use senra_server::{CreateUser, Result, User};

use crate::server::MockServer;

impl MockServer {
    pub async fn create_user(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User> {
        let user_service = self.state.services.user.clone();

        let new_user = CreateUser {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        user_service.create_user(new_user).await
    }

    pub async fn create_token(&mut self, user_id: i64) -> Result<String> {
        let auth_service = self.state.services.auth.clone();

        auth_service.generate_token(user_id).await
    }
}
