use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        calendar::{Calendar, CalendarPublic, CreateCalendarSchema},
    },
    state::AppState,
    middleware::auth::AuthUser,
    utils::{google_oauth::{self, GoogleCalendarListEntry}, auth_helpers::require_admin, jwt::verify_jwt},
};

pub async fn list_calendars(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<Vec<CalendarPublic>>, AppError> {
    let calendars = query_as::<_, CalendarPublic>(
        "SELECT id, name, color, created_at FROM calendars ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(calendars))
}

pub async fn list_google_calendars(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<GoogleCalendarListEntry>>, AppError> {
    require_admin(&auth)?;

    let access_token = google_oauth::get_valid_access_token(&state.db, &state)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get access token: {}", e)))?;

    let calendars = google_oauth::list_calendars(&access_token)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to list Google Calendars: {}", e)))?;

    Ok(Json(calendars))
}

pub async fn create_calendar(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateCalendarSchema>,
) -> Result<Json<Calendar>, AppError> {
    require_admin(&auth)?;

    if payload.url.is_none() && payload.google_id.is_none() {
        return Err(AppError::InvalidInput("Must provide either 'url' or 'google_id'".to_string()));
    }

    if let Some(url) = &payload.url {
        let parsed_url = url::Url::parse(url)
            .map_err(|_| AppError::InvalidInput("Invalid calendar URL format".to_string()))?;

        if parsed_url.scheme() != "https" {
            return Err(AppError::InvalidInput("Only HTTPS URLs are allowed".to_string()));
        }

        let allowed_hosts = ["calendar.google.com", "outlook.live.com", "outlook.office365.com"];
        if let Some(host) = parsed_url.host_str() {
            if !allowed_hosts.contains(&host) {
                return Err(AppError::InvalidInput(
                    "Calendar URL must be from Google Calendar or Outlook".to_string()
                ));
            }
        } else {
            return Err(AppError::InvalidInput("Invalid calendar URL".to_string()));
        }
    }

    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO calendars (id, name, url, google_id, color)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.url)
    .bind(payload.google_id)
    .bind(payload.color.unwrap_or_else(|| "primary".to_string()))
    .execute(&state.db)
    .await?;

    let calendar = query_as::<_, Calendar>("SELECT * FROM calendars WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(calendar))
}

pub async fn delete_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    let result = sqlx::query("DELETE FROM calendars WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::InvalidInput("Calendar not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_calendar_feed(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    // Basic validation that user is logged in
    let auth_header = headers.get(axum::http::header::AUTHORIZATION)
        .ok_or(AppError::AuthError)?;
    let auth_str = auth_header.to_str().map_err(|_| AppError::AuthError)?;
    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::AuthError);
    }
    
    let jwt_secret = state.jwt_secret.read().await;
    let _claims = verify_jwt(&auth_str[7..], jwt_secret.as_bytes())?;

    let calendar = query_as::<_, Calendar>(
        "SELECT * FROM calendars WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::InvalidInput("Calendar not found".to_string()))?;

    if let Some(google_id) = &calendar.google_id {
        if let Some((cached_events, cached_at)) = sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>)>(
            "SELECT events, fetched_at FROM google_calendar_cache WHERE calendar_id = $1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        {
            let cache_age = chrono::Utc::now() - cached_at;
            if cache_age.num_minutes() < 10 {
                let events: serde_json::Value = serde_json::from_str(&cached_events)
                    .unwrap_or(serde_json::json!([]));
                return Ok(Json(events).into_response());
            }
        }

        let access_token = google_oauth::get_valid_access_token(&state.db, &state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to get access token: {}", e)))?;

        let events = google_oauth::get_calendar_events(&access_token, google_id)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to fetch events: {}", e)))?;

        let events_json = serde_json::to_string(&events)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize events: {}", e)))?;
        
        let _ = sqlx::query(
            r#"
            INSERT INTO google_calendar_cache (calendar_id, fetched_at, events)
            VALUES ($1, datetime('now'), $2)
            ON CONFLICT (calendar_id) DO UPDATE
            SET fetched_at = datetime('now'), events = EXCLUDED.events
            "#,
        )
        .bind(id)
        .bind(&events_json)
        .execute(&state.db)
        .await;

        return Ok(Json(events).into_response());
    }

    if let Some(url) = &calendar.url {
        if let Some((feed,)) = sqlx::query_as::<_, (String,)>(
            "SELECT ics_data FROM calendar_feed_cache WHERE calendar_id = $1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        {
            return Ok((
                [(axum::http::header::CONTENT_TYPE, "text/calendar")],
                feed,
            )
                .into_response());
        }

        let client = &state.http_client;
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|_| AppError::InvalidInput("Failed to fetch calendar from provider".to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::InvalidInput("Calendar provider returned an error".to_string()));
        }

        let body = resp
            .text()
            .await
            .map_err(|_| AppError::InvalidInput("Failed to read calendar body".to_string()))?;

        let _ = sqlx::query(
            r#"
            INSERT INTO calendar_feed_cache (calendar_id, fetched_at, ics_data)
            VALUES ($1, datetime('now'), $2)
            ON CONFLICT (calendar_id) DO UPDATE
            SET fetched_at = datetime('now'), ics_data = EXCLUDED.ics_data
            "#,
        )
        .bind(id)
        .bind(&body)
        .execute(&state.db)
        .await;

        return Ok((
            [(axum::http::header::CONTENT_TYPE, "text/calendar")],
            body,
        )
            .into_response());
    }

    Err(AppError::InvalidInput("Calendar has no URL or ID".to_string()))
}