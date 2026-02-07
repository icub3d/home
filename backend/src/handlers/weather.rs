use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{error::AppError, state::AppState, middleware::auth::AuthUser};

pub async fn get_weather(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<Option<serde_json::Value>>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT data FROM weather_cache LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some((data,)) = row {
        let json: serde_json::Value = serde_json::from_str(&data)
            .map_err(|_| AppError::BadRequest("Failed to parse cached weather data".to_string()))?;
        Ok(Json(Some(json)))
    } else {
        Ok(Json(None))
    }
}