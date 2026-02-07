use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc, Duration};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::{
    error::AppError,
    state::{AppState, CachedPhotos},
    utils::auth_helpers::{require_admin, generate_random_token},
    middleware::auth::AuthUser,
    utils::google_oauth::{self, PickerSession},
    utils::jwt::verify_jwt,
};

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthStartResponse {
    pub auth_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmSelectionRequest {
    pub session_id: String,
}

pub async fn get_photo(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    // Basic validation that requester is authenticated (either via JWT or display token)
    let authenticated = if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let jwt_secret = state.jwt_secret.read().await;
                verify_jwt(&auth_str[7..], jwt_secret.as_bytes()).is_ok()
            } else { false }
        } else { false }
    } else if let Some(token) = params.get("token") {
        let token_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM display_tokens WHERE token = $1)"
        )
        .bind(token)
        .fetch_one(&state.db)
        .await?;
        token_exists
    } else {
        false
    };

    if !authenticated {
        return Err(AppError::AuthError);
    }

    // Security check for filename to prevent path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::BadRequest("Invalid filename".to_string()));
    }

    let file_path = state.photos_dir.join(&filename);

    if !file_path.exists() {
        return Err(AppError::BadRequest("Photo not found".to_string()));
    }

    let content_type = if filename.to_lowercase().ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };

    let bytes: Vec<u8> = tokio::fs::read(&file_path).await
        .map_err(|_| AppError::BadRequest("Failed to read photo".to_string()))?;

    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        bytes,
    ))
}

async fn download_file(client: &reqwest::Client, url: &str, file_path: &std::path::Path, access_token: &str) -> Result<(), String> {
    let response = client.get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    let mut file = fs::File::create(file_path).await.map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn start_google_oauth(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<OAuthStartResponse>, AppError> {
    require_admin(&auth)?;
    
    let google_client_id = state.google_client_id.read().await;
    let google_oauth_redirect_uri = state.google_oauth_redirect_uri.read().await;

    if google_client_id.is_empty() {
        return Err(AppError::BadRequest("Google OAuth not configured.".to_string()));
    }

    let state_token = generate_random_token(32);

    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('google_oauth_state', $1) 
         ON CONFLICT (key) DO UPDATE SET value = $1"
    )
        .bind(&state_token)
        .execute(&state.db)
        .await?;

    let auth_url = google_oauth::build_auth_url(
        &google_client_id, 
        &google_oauth_redirect_uri, 
        &state_token
    );

    Ok(Json(OAuthStartResponse { auth_url }))
}

pub async fn google_oauth_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Verify state token
    let stored_state: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'google_oauth_state'"
    )
        .fetch_optional(&state.db)
        .await?;

    if stored_state.as_deref() != Some(&params.state) {
        return Err(AppError::BadRequest("Invalid OAuth state".to_string()));
    }

    let google_client_id = state.google_client_id.read().await;
    let google_client_secret = state.google_client_secret.read().await;
    let google_oauth_redirect_uri = state.google_oauth_redirect_uri.read().await;

    // Exchange code for tokens
    let token_response = google_oauth::exchange_code_for_tokens(
        &google_client_id,
        &google_client_secret,
        &params.code,
        &google_oauth_redirect_uri,
    )
    .await
    .map_err(|e| AppError::BadRequest(format!("Failed to exchange OAuth code: {}", e)))?;

    // Store access token
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('google_photos_access_token', $1) 
         ON CONFLICT (key) DO UPDATE SET value = $1"
    )
        .bind(&token_response.access_token)
        .execute(&state.db)
        .await?;

    if let Some(refresh_token) = &token_response.refresh_token {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ('google_photos_refresh_token', $1) 
             ON CONFLICT (key) DO UPDATE SET value = $1"
        )
            .bind(refresh_token)
            .execute(&state.db)
            .await?;
    }

    let expiry = Utc::now() + Duration::seconds(token_response.expires_in);
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('google_photos_token_expiry', $1) 
         ON CONFLICT (key) DO UPDATE SET value = $1"
    )
        .bind(expiry.to_rfc3339())
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM settings WHERE key = 'google_oauth_state'")
        .execute(&state.db)
        .await?;

    let base_url = state.base_url.read().await;
    Ok(Redirect::to(&format!("{}/settings?google_photos=connected", base_url.trim_end_matches('/'))))
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<PickerSession>, AppError> {
    require_admin(&auth)?;

    let access_token = google_oauth::get_valid_access_token(&state.db, &state)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get access token: {}", e)))?;

    let session = google_oauth::create_picker_session(&access_token)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create picker session: {}", e)))?;

    Ok(Json(session))
}

pub async fn confirm_selection(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<ConfirmSelectionRequest>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    let access_token = google_oauth::get_valid_access_token(&state.db, &state)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get access token: {}", e)))?;

    let items = google_oauth::list_picked_media_items(&access_token, &payload.session_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to list picked items: {}", e)))?;

    if items.is_empty() {
        return Err(AppError::BadRequest("No items were selected.".to_string()));
    }

    let state_clone = state.clone();
    let access_token_clone = access_token.clone();
    
    tokio::spawn(async move {
        let photos_dir = state_clone.photos_dir.clone();
        let _ = fs::create_dir_all(&photos_dir).await;

        let mut saved_urls: Vec<String> = Vec::new();

        for item in items {
            let url = format!("{}=w1920-h1080", item.media_file.base_url);
            let extension = if item.media_file.filename.to_lowercase().ends_with(".png") { "png" } else { "jpg" };
            let filename = format!("{}.{}", item.id, extension);
            let file_path = photos_dir.join(&filename);
            
            if download_file(&state_clone.http_client, &url, &file_path, &access_token_clone).await.is_ok() {
                saved_urls.push(format!("/api/photos/{}", filename));
            }
        }

        if !saved_urls.is_empty() {
            if let Ok(urls_json) = serde_json::to_string(&saved_urls) {
                let _ = sqlx::query(
                    "INSERT INTO settings (key, value) VALUES ('google_photos_picked_items', $1) 
                     ON CONFLICT (key) DO UPDATE SET value = $1"
                )
                    .bind(&urls_json)
                    .execute(&state_clone.db)
                    .await;
            }

            let mut cache = state_clone.photo_cache.write().await;
            *cache = Some(CachedPhotos {
                source_url: "google_photos_picker".to_string(),
                images: saved_urls,
                last_updated: Utc::now(),
            });
        }
    });

    Ok(StatusCode::ACCEPTED)
}

pub async fn disconnect_google_photos(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    sqlx::query(
        "DELETE FROM settings WHERE key IN (
            'google_photos_access_token', 'google_photos_refresh_token', 
            'google_photos_token_expiry', 'google_photos_album_id', 
            'google_photos_picked_items'
        )"
    )
        .execute(&state.db)
        .await?;

    let mut cache = state.photo_cache.write().await;
    *cache = None;
    
    let photos_dir = state.photos_dir.clone();
    if photos_dir.exists() {
        let _ = fs::remove_dir_all(photos_dir).await;
    }

    Ok(StatusCode::NO_CONTENT)
}