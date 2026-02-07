use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::AppState,
    utils::jwt::verify_jwt,
    models::user::UserRole,
};

/// Authenticated user extractor - contains JWT claims
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: UserRole,
}

impl AuthUser {
    /// Check if user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or(AppError::AuthError)?;

        let auth_header = auth_header.to_str().map_err(|_| AppError::AuthError)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::AuthError);
        }

        let token = &auth_header[7..];

        let jwt_secret = state.jwt_secret.read().await;
        let claims = verify_jwt(token, jwt_secret.as_bytes())?;
        
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::AuthError)?;
        let role = claims.role.parse::<UserRole>()
            .map_err(|_| AppError::AuthError)?;

        Ok(AuthUser {
            user_id,
            role,
        })
    }
}
