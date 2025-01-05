use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;

use crate::errors::{AppError, AuthError};
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
            .ok_or(AuthError::InvalidCredentials)?;

        let user_id = state.services.auth.authorize(&token).await?;

        Ok(AuthUser { user_id })
    }
}

impl<S> OptionalFromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let state = AppState::from_ref(state);
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        match token {
            Some(token) => {
                let user_id = state.services.auth.authorize(&token).await?;
                Ok(Some(AuthUser { user_id }))
            }
            None => Ok(None),
        }
    }
}
