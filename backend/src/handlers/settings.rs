use axum::{Json, extract::State};
use std::sync::Arc;
use regex::Regex;
use url::Url;

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    models::settings::{AppSettings, Setting, UpdateAppSettingsSchema},
    state::AppState,
    utils::auth_helpers::require_admin,
};

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<AppSettings>, AppError> {
    require_admin(&auth)?;

    let rows = sqlx::query_as::<_, Setting>(
        "SELECT key, value, updated_at FROM settings",
    )
    .fetch_all(&state.db)
    .await?;

    let mut settings = AppSettings::default();
    for row in rows {
        match row.key.as_str() {
            "family_name" => settings.family_name = row.value,
            "base_url" => settings.base_url = row.value,
            "weather_zip_code" => settings.weather_zip_code = row.value,
            "background_url" => settings.background_url = row.value,
            "openweather_api_key" => settings.openweather_api_key = row.value,
            "google_client_id" => settings.google_client_id = row.value,
            "google_client_secret" => settings.google_client_secret = row.value,
            "google_photos_access_token" => settings.google_photos_access_token = row.value,
            "google_photos_refresh_token" => {
                if !row.value.is_empty() {
                    settings.google_connected = true;
                }
                settings.google_photos_refresh_token = row.value;
            }
            "google_photos_token_expiry" => settings.google_photos_token_expiry = row.value,
            "google_photos_album_id" => settings.google_photos_album_id = row.value,
            "google_photos_picked_items" => settings.google_photos_picked_items = row.value,
            "last_background_refresh" => settings.last_background_refresh = row.value,
            _ => {}
        }
    }

    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateAppSettingsSchema>,
) -> Result<Json<AppSettings>, AppError> {
    require_admin(&auth)?;

    if let Some(name) = payload.family_name {
        if name.len() > 100 {
            return Err(AppError::InvalidInput("Family name too long".to_string()));
        }
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("family_name")
        .bind(name)
        .execute(&state.db)
        .await?;
    }

    if let Some(url) = payload.base_url {
        if url.len() > 2048 {
            return Err(AppError::InvalidInput("Base URL too long".to_string()));
        }
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("base_url")
        .bind(&url)
        .execute(&state.db)
        .await?;

        // Also update redirect URI
        let redirect_uri = format!("{}/api/google-photos/callback", url.trim_end_matches('/'));
        *state.google_oauth_redirect_uri.write().await = redirect_uri.clone();
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("google_oauth_redirect_uri")
        .bind(redirect_uri)
        .execute(&state.db)
        .await?;
    }

    if let Some(zip) = payload.weather_zip_code {
        static ZIP_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let zip_regex = ZIP_REGEX.get_or_init(|| Regex::new(r"^\d{5}$").expect("Invalid regex"));
        
        if !zip.is_empty() && !zip_regex.is_match(&zip) {
             return Err(AppError::InvalidInput("Invalid zip code format".to_string()));
        }

        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("weather_zip_code")
        .bind(zip)
        .execute(&state.db)
        .await?;
    }

    if let Some(url) = payload.background_url {
        if !url.is_empty() {
             let parsed_url = Url::parse(&url)
                 .map_err(|_| AppError::InvalidInput("Invalid URL format".to_string()))?;
             if parsed_url.scheme() != "https" {
                 return Err(AppError::InvalidInput("Only HTTPS URLs are allowed".to_string()));
             }
        }

        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("background_url")
        .bind(url)
        .execute(&state.db)
        .await?;
    }

    if let Some(key) = payload.openweather_api_key {
        if key.len() > 255 {
            return Err(AppError::InvalidInput("API key too long".to_string()));
        }
        *state.openweather_api_key.write().await = key.clone();
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("openweather_api_key")
        .bind(key)
        .execute(&state.db)
        .await?;
    }

    if let Some(id) = payload.google_client_id {
        if id.len() > 255 {
            return Err(AppError::InvalidInput("Client ID too long".to_string()));
        }
        *state.google_client_id.write().await = id.clone();
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("google_client_id")
        .bind(id)
        .execute(&state.db)
        .await?;
    }

    if let Some(secret) = payload.google_client_secret {
        if secret.len() > 255 {
            return Err(AppError::InvalidInput("Client secret too long".to_string()));
        }
        *state.google_client_secret.write().await = secret.clone();
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES ($1, $2) 
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = datetime('now')"
        )
        .bind("google_client_secret")
        .bind(secret)
        .execute(&state.db)
        .await?;
    }

    get_settings(State(state), auth).await
}