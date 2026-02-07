use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;

use crate::{
    error::AppError,
    models::{
        display::{DisplayToken, CreateTokenSchema, DisplayData},
        user::UserBalance,
        calendar::CalendarPublic,
        chore::ChoreWithUser,
    },
    state::{AppState, CachedPhotos},
    utils::auth_helpers::{require_admin, generate_random_token},
    middleware::auth::AuthUser,
};

pub async fn list_tokens(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<DisplayToken>>, AppError> {
    require_admin(&auth)?;

    let tokens = query_as::<_, DisplayToken>(
        "SELECT * FROM display_tokens ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Json(tokens))
}

pub async fn create_token(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateTokenSchema>,
) -> Result<Json<DisplayToken>, AppError> {
    require_admin(&auth)?;

    let token_string = generate_random_token(32);
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO display_tokens (id, name, token) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(payload.name)
    .bind(token_string)
    .execute(&state.db)
    .await?;

    let token = query_as::<_, DisplayToken>("SELECT * FROM display_tokens WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(token))
}

pub async fn delete_token(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    sqlx::query("DELETE FROM display_tokens WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_display_data(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<DisplayData>, AppError> {
    let token_header = headers.get("X-Display-Token")
        .ok_or(AppError::AuthError)?
        .to_str()
        .map_err(|_| AppError::AuthError)?;

    let _token_record = query_as::<_, DisplayToken>("SELECT * FROM display_tokens WHERE token = $1")
        .bind(token_header)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::AuthError)?;

    let weather: Option<String> = sqlx::query_scalar(
        "SELECT data FROM weather_cache LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;
    let weather_json = weather.and_then(|s| serde_json::from_str(&s).ok());

    let calendars = query_as::<_, CalendarPublic>(
        "SELECT id, name, color, created_at FROM calendars ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let allowances = query_as::<_, UserBalance>(
        r#"
        SELECT u.id as user_id, u.name, COALESCE(
            (SELECT balance FROM allowance_ledger 
             WHERE user_id = u.id 
             ORDER BY created_at DESC 
             LIMIT 1), 0) as balance
        FROM users u
        WHERE u.track_allowance = 1
        ORDER BY u.name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let chores = query_as::<_, ChoreWithUser>(
        r#"
        SELECT c.id, c.description, c.assigned_to, u.name as assigned_name,
               c.reward, c.completed, c.created_at, c.updated_at
        FROM chores c
        JOIN users u ON c.assigned_to = u.id
        WHERE c.completed = 0
        ORDER BY u.name ASC, c.created_at ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let mut background_url = None;

    let picked_items_json: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'google_photos_picked_items'"
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(json_str) = picked_items_json {
        if !json_str.is_empty() && json_str != "[]" {
             if let Ok(urls) = serde_json::from_str::<Vec<String>>(&json_str) {
                 if !urls.is_empty() {
                    let mut cache_lock = state.photo_cache.write().await;
                    let needs_update = match &*cache_lock {
                        Some(cache) => cache.source_url != "google_photos_picker" || cache.images.len() != urls.len(),
                        None => true,
                    };

                    if needs_update {
                        *cache_lock = Some(CachedPhotos {
                            source_url: "google_photos_picker".to_string(),
                            images: urls.clone(),
                            last_updated: Utc::now(),
                        });
                    }

                    if let Some(cache) = &*cache_lock {
                        if !cache.images.is_empty() {
                            let idx = rand::rng().random_range(0..cache.images.len());
                            background_url = Some(cache.images[idx].clone());
                        }
                    }
                 }
             }
        }
    } else {
        let bg_setting: Option<String> = sqlx::query_scalar(
            "SELECT value FROM settings WHERE key = 'background_url'"
        )
        .fetch_optional(&state.db)
        .await?;

        if let Some(url) = bg_setting {
            if url.contains("photos.app.goo.gl") || url.contains("photos.google.com") {
                let mut cache_lock = state.photo_cache.write().await;
                let needs_refresh = match &*cache_lock {
                    Some(cache) => cache.source_url != url || (Utc::now() - cache.last_updated).num_hours() > 24,
                    None => true,
                };

                if needs_refresh {
                    match crate::utils::google_photos::extract_photos(&url).await {
                        Ok(photos) => {
                            if !photos.is_empty() {
                                *cache_lock = Some(CachedPhotos {
                                    source_url: url.clone(),
                                    images: photos,
                                    last_updated: Utc::now(),
                                });
                            }
                        },
                        Err(e) => {
                            tracing::error!("Failed to scrape Google Photos: {}", e);
                        }
                    }
                }
                
                if let Some(cache) = &*cache_lock {
                    if !cache.images.is_empty() {
                        let idx = rand::rng().random_range(0..cache.images.len());
                        background_url = Some(cache.images[idx].clone());
                    }
                }
            } else {
                background_url = Some(url);
            }
        }
    }

    Ok(Json(DisplayData {
        weather: weather_json,
        calendars,
        allowances,
        chores,
        background_url,
    }))
}
