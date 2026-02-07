use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::user::{User, LoginResponse, UserLoginSchema},
    state::AppState,
    utils::jwt::{verify_password, create_jwt, hash_password},
    utils::auth_helpers::generate_random_token,
};

#[derive(Debug, Deserialize)]
pub struct SetupSchema {
    pub username: String,
    pub name: String,
    pub password: String,
    pub family_name: String,
    pub base_url: String,
    pub openweather_api_key: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub initialized: bool,
}

pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SystemStatus>, AppError> {
    let existing_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM users"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(Json(SystemStatus { initialized: existing_count > 0 }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserLoginSchema>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1"
    )
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Sqlx)?
        .ok_or(AppError::LoginFailed)?;

    if verify_password(&payload.password, &user.password_hash)? {
        let jwt_secret = state.jwt_secret.read().await;
        let token = create_jwt(user.id, &user.role, jwt_secret.as_bytes())?;
        Ok(Json(LoginResponse { token, user }))
    } else {
        Err(AppError::LoginFailed)
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<SetupSchema>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<LoginResponse>, AppError> {
    let Json(payload) = match payload {
        Ok(p) => p,
        Err(rejection) => {
            tracing::error!("JSON rejection in register: {}", rejection.body_text());
            return Err(AppError::BadRequest(rejection.body_text()));
        }
    };

    tracing::info!("Received setup request for family: {}", payload.family_name);
    
    // Check if any users exist
    let existing_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM users"
    )
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Error checking existing users: {:?}", e);
            AppError::Sqlx(e)
        })?;

    if existing_count > 0 {
        tracing::warn!("Setup attempted but users already exist (count: {})", existing_count);
        return Err(AppError::InvalidInput("Setup is already complete".to_string()));
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        tracing::error!("Error starting setup transaction: {:?}", e);
        AppError::Sqlx(e)
    })?;

    // Generate JWT Secret if not already set
    let mut jwt_secret_lock = state.jwt_secret.write().await;
    if jwt_secret_lock.is_empty() {
        tracing::info!("Generating new JWT secret");
        let new_secret = generate_random_token(64);
        *jwt_secret_lock = new_secret.clone();
        sqlx::query("INSERT INTO settings (key, value) VALUES ('jwt_secret', $1)")
            .bind(&new_secret)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Error saving jwt_secret: {:?}", e);
                AppError::Sqlx(e)
            })?;
    }
    let current_jwt_secret = jwt_secret_lock.clone();
    drop(jwt_secret_lock);

    // Save Family Name
    sqlx::query("INSERT INTO settings (key, value) VALUES ('family_name', $1)")
        .bind(&payload.family_name)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error saving family_name: {:?}", e);
            AppError::Sqlx(e)
        })?;

    // Save Base URL
    *state.base_url.write().await = payload.base_url.clone();
    sqlx::query("INSERT INTO settings (key, value) VALUES ('base_url', $1)")
        .bind(&payload.base_url)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error saving base_url: {:?}", e);
            AppError::Sqlx(e)
        })?;

    // Set default Google OAuth Redirect URI based on Base URL
    let redirect_uri = format!("{}/api/google-photos/callback", payload.base_url.trim_end_matches('/'));
    *state.google_oauth_redirect_uri.write().await = redirect_uri.clone();
    sqlx::query("INSERT INTO settings (key, value) VALUES ('google_oauth_redirect_uri', $1) ON CONFLICT(key) DO UPDATE SET value = $1")
        .bind(&redirect_uri)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error saving redirect_uri: {:?}", e);
            AppError::Sqlx(e)
        })?;

    // Save other settings
    if let Some(key) = payload.openweather_api_key {
        if !key.is_empty() {
            *state.openweather_api_key.write().await = key.clone();
            sqlx::query("INSERT INTO settings (key, value) VALUES ('openweather_api_key', $1) ON CONFLICT(key) DO UPDATE SET value = $1")
                .bind(&key)
                .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
        }
    }

    if let Some(id) = payload.google_client_id {
        if !id.is_empty() {
            *state.google_client_id.write().await = id.clone();
            sqlx::query("INSERT INTO settings (key, value) VALUES ('google_client_id', $1) ON CONFLICT(key) DO UPDATE SET value = $1")
                .bind(&id)
                .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
        }
    }

    if let Some(secret) = payload.google_client_secret {
        if !secret.is_empty() {
            *state.google_client_secret.write().await = secret.clone();
            sqlx::query("INSERT INTO settings (key, value) VALUES ('google_client_secret', $1) ON CONFLICT(key) DO UPDATE SET value = $1")
                .bind(&secret)
                .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
        }
    }

    // Create admin user
    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&payload.password)?;
    
    tracing::info!("Creating admin user: {}", payload.username);
    sqlx::query(
        r#"
        INSERT INTO users (id, username, name, password_hash, role, track_allowance)
        VALUES ($1, $2, $3, $4, 'admin', 0)
        "#
    )
        .bind(user_id)
        .bind(&payload.username)
        .bind(&payload.name)
        .bind(&password_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Error creating admin user: {:?}", e);
            AppError::Sqlx(e)
        })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Error committing setup transaction: {:?}", e);
        AppError::Sqlx(e)
    })?;

    // Fetch the created user
    let user = query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Sqlx)?;

    // Create JWT token
    let token = create_jwt(user.id, &user.role, current_jwt_secret.as_bytes())?;

    tracing::info!("Setup complete for family: {}", payload.family_name);
    Ok(Json(LoginResponse { token, user }))
}
