use std::{sync::Arc, time::Duration};

use sqlx::query_as;

use crate::{error::AppError, models::calendar::Calendar, state::AppState, utils::google_oauth};

const DEFAULT_REFRESH_SECONDS: u64 = 60 * 60;

async fn refresh_interval_seconds() -> u64 {
    std::env::var("REFRESH_INTERVAL_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_REFRESH_SECONDS)
        .max(10)
}

pub fn start_refresh_loop(state: Arc<AppState>) {
    tokio::spawn(async move {
        let secs = refresh_interval_seconds().await;

        // Initial refresh on startup
        tracing::info!("Initial background refresh cycle (interval: {}s)", secs);

        if let Err(e) = refresh_all(&state).await {
            tracing::warn!(error = ?e, "initial refresh failed");
        }

        loop {
            tokio::time::sleep(Duration::from_secs(secs)).await;

            let secs = refresh_interval_seconds().await;
            tracing::info!("Starting background refresh cycle (interval: {}s)", secs);

            if let Err(e) = refresh_all(&state).await {
                tracing::warn!(error = ?e, "refresh cycle failed");
            }

            tracing::info!("Background refresh cycle complete");
        }
    });
}

async fn refresh_all(state: &AppState) -> Result<(), AppError> {
    if let Err(e) = refresh_weather(state).await {
        tracing::warn!(error = ?e, "weather refresh failed");
    }

    if let Err(e) = refresh_calendar_feeds(state).await {
        tracing::warn!(error = ?e, "calendar refresh failed");
    }

    Ok(())
}

async fn refresh_weather(state: &AppState) -> Result<(), AppError> {
    let zip: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'weather_zip_code'",
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(zip) = zip else {
        tracing::debug!("Weather zip code not configured, skipping refresh");
        return Ok(());
    };

    let api_key = state.openweather_api_key.read().await;
    if api_key.is_empty() {
        tracing::debug!("OpenWeather API key not configured, skipping refresh");
        return Ok(());
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?zip={},us&units=imperial&appid={}",
        zip, *api_key
    );

    let resp = state.http_client.get(url).send().await.map_err(|e| {
        tracing::warn!(error = ?e, "Weather request failed");
        AppError::InvalidInput("Failed to fetch weather from provider".to_string())
    })?;

    if !resp.status().is_success() {
        return Err(AppError::InvalidInput(
            "Weather provider returned an error".to_string(),
        ));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| AppError::InvalidInput("Failed to parse weather JSON".to_string()))?;

    let data_json = serde_json::to_string(&data).unwrap_or_default();

    sqlx::query(
        r#"
        INSERT INTO weather_cache (id, fetched_at, data)
        VALUES (1, datetime('now'), $1)
        ON CONFLICT (id) DO UPDATE
        SET fetched_at = datetime('now'), data = EXCLUDED.data
        "#,
    )
    .bind(&data_json)
    .execute(&state.db)
    .await?;

    tracing::info!("Weather cache updated");
    Ok(())
}

async fn refresh_calendar_feeds(state: &AppState) -> Result<(), AppError> {
    let calendars = query_as::<_, Calendar>(
        "SELECT * FROM calendars ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    if calendars.is_empty() {
        return Ok(());
    }

    let mut google_count = 0;
    let mut ical_count = 0;

    for cal in calendars {
        if let Some(google_id) = &cal.google_id {
            match google_oauth::get_valid_access_token(&state.db, state).await {
                Ok(access_token) => {
                    match google_oauth::get_calendar_events(&access_token, google_id).await {
                        Ok(events) => {
                            if let Ok(events_json) = serde_json::to_string(&events) {
                                let _ = sqlx::query(
                                    r#"
                                    INSERT INTO google_calendar_cache (calendar_id, fetched_at, events)
                                    VALUES ($1, datetime('now'), $2)
                                    ON CONFLICT (calendar_id) DO UPDATE
                                    SET fetched_at = datetime('now'), events = EXCLUDED.events
                                    "#,
                                )
                                .bind(cal.id)
                                .bind(events_json)
                                .execute(&state.db)
                                .await;
                                google_count += 1;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(calendar_id = %cal.id, google_id = %google_id, error = ?e, "failed fetching Google calendar");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "failed to get Google access token for calendar refresh");
                }
            }
            continue;
        }

        let url = match &cal.url {
            Some(u) => u,
            None => continue,
        };

        let resp = match state.http_client.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(calendar_id = %cal.id, error = ?e, "failed fetching calendar");
                continue;
            }
        };

        if !resp.status().is_success() {
            continue;
        }

        let feed = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(calendar_id = %cal.id, error = ?e, "failed reading calendar body");
                continue;
            }
        };

        let _ = sqlx::query(
            r#"
            INSERT INTO calendar_feed_cache (calendar_id, fetched_at, ics_data)
            VALUES ($1, datetime('now'), $2)
            ON CONFLICT (calendar_id) DO UPDATE
            SET fetched_at = datetime('now'), ics_data = EXCLUDED.ics_data
            "#,
        )
        .bind(cal.id)
        .bind(&feed)
        .execute(&state.db)
        .await;

        ical_count += 1;
    }

    tracing::info!(google = google_count, ical = ical_count, "Calendar refresh complete");

    Ok(())
}
