use senra_server::models::{CreateUser, User};

use crate::server::MockServer;

impl MockServer {
    pub async fn create_test_user(&mut self, username: &str, password: &str) -> User {
        let email = format!("{}@example.com", username);

        let user_service = self.state.services.user.clone();

        let new_user = CreateUser {
            username: username.to_string(),
            email,
            password: password.to_string(),
        };

        let user = user_service.create_user(new_user).await.unwrap();

        user
    }

    pub async fn create_test_token(&mut self, user_id: i64) -> String {
        let auth_service = self.state.services.auth.clone();

        let token = auth_service.generate_token(user_id).await.unwrap();

        token
    }
}
