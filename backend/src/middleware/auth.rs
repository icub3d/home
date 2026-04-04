use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use std::sync::Arc;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use sqlx::FromRow;

use crate::{
    error::AppError,
    state::AppState,
    utils::jwt::verify_jwt,
    models::user::UserRole,
};

#[derive(FromRow)]
struct ApiKeyUserInfo {
    id: Uuid,
    role: UserRole,
}

/// Authenticated user extractor - contains JWT claims or API Key info
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: UserRole,
    pub from_api_key: bool,
}

impl AuthUser {
    /// Check if user has admin privileges. 
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Explicit check for whether this user is allowed to modify system settings.
    /// API Keys are NOT allowed to perform these actions.
    pub fn can_manage_settings(&self) -> bool {
        !self.from_api_key && self.is_admin()
    }
}

async fn validate_api_key(key: &str, state: &Arc<AppState>) -> Result<Option<AuthUser>, AppError> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    let user_info: Option<ApiKeyUserInfo> = sqlx::query_as::<sqlx::Sqlite, ApiKeyUserInfo>(
        "SELECT u.id, u.role FROM users u JOIN api_keys ak ON u.id = ak.user_id WHERE ak.key_hash = $1"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Sqlx)?;

    if let Some(info) = user_info {
        // Update last used
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = datetime('now') WHERE key_hash = $1")
            .bind(&key_hash)
            .execute(&state.db)
            .await;

        return Ok(Some(AuthUser {
            user_id: info.id,
            role: info.role,
            from_api_key: true,
        }));
    }
    Ok(None)
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        // 1. Check for API Key in X-API-Key header
        if let Some(api_key) = parts.headers.get("X-API-Key") {
            let api_key_str = api_key.to_str().map_err(|_| AppError::AuthError)?;
            if let Some(auth_user) = validate_api_key(api_key_str, state).await? {
                return Ok(auth_user);
            }
        }

        // 2. Check Authorization header
        if let Some(auth_header) = parts.headers.get(axum::http::header::AUTHORIZATION) {
            let auth_header_str = auth_header.to_str().map_err(|_| AppError::AuthError)?;

            if auth_header_str.starts_with("Bearer ") {
                let token = &auth_header_str[7..];

                // 2a. Check if Bearer token is actually an API Key (starts with hb_)
                if token.starts_with("hb_") {
                    if let Some(auth_user) = validate_api_key(token, state).await? {
                        return Ok(auth_user);
                    }
                }

                // 2b. Otherwise, parse as JWT
                let jwt_secret = state.jwt_secret.read().await;
                let claims = verify_jwt(token, jwt_secret.as_bytes())?;
                
                let user_id = Uuid::parse_str(&claims.sub)
                    .map_err(|_| AppError::AuthError)?;
                let role = claims.role.parse::<UserRole>()
                    .map_err(|_| AppError::AuthError)?;

                return Ok(AuthUser {
                    user_id,
                    role,
                    from_api_key: false,
                });
            }
        }

        Err(AppError::AuthError)
    }
}
