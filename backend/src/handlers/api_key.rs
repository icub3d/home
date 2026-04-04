use axum::extract::{State, Path};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;
use sqlx::query_as;

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    models::api_key::{ApiKey, CreateApiKeySchema, CreatedApiKeyResponse},
    state::AppState,
    utils::auth_helpers::{require_admin, generate_random_token},
};

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<ApiKey>>, AppError> {
    require_admin(&auth)?;

    let keys: Vec<ApiKey> = query_as(
        "SELECT * FROM api_keys ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(keys))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateApiKeySchema>,
) -> Result<Json<CreatedApiKeyResponse>, AppError> {
    require_admin(&auth)?;

    let id = Uuid::new_v4();
    let raw_key = format!("hb_{}", generate_random_token(48));
    
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    sqlx::query(
        "INSERT INTO api_keys (id, key_hash, name, user_id) VALUES ($1, $2, $3, $4)"
    )
    .bind(id)
    .bind(key_hash)
    .bind(payload.name.clone())
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(CreatedApiKeyResponse {
        id,
        key: raw_key,
        name: payload.name,
    }))
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
