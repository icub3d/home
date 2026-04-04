use axum::{Json, extract::State};
use std::sync::Arc;
use regex::Regex;
use url::Url;

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    models::settings::{AppSettings, UpdateAppSettingsSchema},
    state::AppState,
};

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<AppSettings>, AppError> {
    if !auth.is_admin() {
        return Err(AppError::AuthError);
    }

    let family_name = get_setting(&state.db, "family_name").await;
    let base_url = get_setting(&state.db, "base_url").await;
    let weather_zip_code = get_setting(&state.db, "weather_zip_code").await;
    let background_url = get_setting(&state.db, "background_url").await;
    let openweather_api_key = get_setting(&state.db, "openweather_api_key").await;
    let google_client_id = get_setting(&state.db, "google_client_id").await;
    let google_client_secret = get_setting(&state.db, "google_client_secret").await;

    let google_photos_access_token = get_setting(&state.db, "google_access_token").await;
    let google_photos_refresh_token = get_setting(&state.db, "google_refresh_token").await;
    let google_photos_token_expiry = get_setting(&state.db, "google_token_expiry").await;
    let google_photos_album_id = get_setting(&state.db, "google_photos_album_id").await;
    let google_photos_picked_items = get_setting(&state.db, "google_photos_picked_items").await;
    let last_background_refresh = get_setting(&state.db, "last_background_refresh").await;

    let google_connected = !google_client_id.is_empty() && !google_client_secret.is_empty() && 
        !google_photos_refresh_token.is_empty();

    Ok(Json(AppSettings {
        family_name,
        base_url,
        weather_zip_code,
        background_url,
        openweather_api_key,
        google_client_id,
        google_client_secret,
        google_connected,
        google_photos_access_token,
        google_photos_refresh_token,
        google_photos_token_expiry,
        google_photos_album_id,
        google_photos_picked_items,
        last_background_refresh,
    }))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateAppSettingsSchema>,
) -> Result<Json<AppSettings>, AppError> {
    // API Keys are NOT allowed to modify settings
    if !auth.can_manage_settings() {
        return Err(AppError::AuthError);
    }

    // Validation
    if let Some(ref name) = payload.family_name {
        if name.len() > 100 {
            return Err(AppError::InvalidInput("Family name too long".to_string()));
        }
    }

    if let Some(ref url_str) = payload.base_url {
        if url_str.len() > 255 {
            return Err(AppError::InvalidInput("Base URL too long".to_string()));
        }
        Url::parse(url_str).map_err(|_| AppError::InvalidInput("Invalid Base URL".to_string()))?;
    }

    if let Some(ref zip) = payload.weather_zip_code {
        let zip_regex = Regex::new(r"^\d{5}$").unwrap();
        if !zip_regex.is_match(zip) && !zip.is_empty() {
            return Err(AppError::InvalidInput("Invalid zip code".to_string()));
        }
    }

    // Save settings
    if let Some(val) = payload.family_name {
        save_setting(&state.db, "family_name", &val).await?;
    }
    if let Some(val) = payload.base_url {
        save_setting(&state.db, "base_url", &val).await?;
        *state.base_url.write().await = val;
    }
    if let Some(val) = payload.weather_zip_code {
        save_setting(&state.db, "weather_zip_code", &val).await?;
    }
    if let Some(val) = payload.background_url {
        save_setting(&state.db, "background_url", &val).await?;
    }
    if let Some(val) = payload.openweather_api_key {
        save_setting(&state.db, "openweather_api_key", &val).await?;
        *state.openweather_api_key.write().await = val;
    }
    if let Some(val) = payload.google_client_id {
        save_setting(&state.db, "google_client_id", &val).await?;
        *state.google_client_id.write().await = val;
    }
    if let Some(val) = payload.google_client_secret {
        save_setting(&state.db, "google_client_secret", &val).await?;
        *state.google_client_secret.write().await = val;
    }

    get_settings(State(state), auth).await
}

async fn get_setting(pool: &sqlx::SqlitePool, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .unwrap_or_default()
}

async fn save_setting(pool: &sqlx::SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query("INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await
        .map_err(AppError::Sqlx)?;
    Ok(())
}
