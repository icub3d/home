use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use sqlx::query_as;

use crate::{
    error::AppError,
    models::{
        backup::BackupData,
        user::{BackupUser, AllowanceTransaction, UserRole},
        settings::Setting,
        calendar::Calendar,
    },
    state::AppState,
    utils::auth_helpers::require_admin,
    middleware::auth::AuthUser,
};

pub async fn export_backup(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    require_admin(&auth)?;

    let users = query_as::<_, BackupUser>("SELECT * FROM users")
        .fetch_all(&state.db).await?;
    let settings = query_as::<_, Setting>("SELECT * FROM settings")
        .fetch_all(&state.db).await?;
    let calendars = query_as::<_, Calendar>("SELECT * FROM calendars")
        .fetch_all(&state.db).await?;
    let ledger = query_as::<_, AllowanceTransaction>("SELECT * FROM allowance_ledger")
        .fetch_all(&state.db).await?;

    let backup = BackupData {
        users,
        settings,
        calendars,
        allowance_ledger: ledger,
        version: 1,
        created_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string_pretty(&backup).map_err(|_| AppError::InvalidInput("Failed to serialize backup".to_string()))?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"home_backup.json\""),
        ],
        json,
    ).into_response())
}

pub async fn import_backup(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(backup): Json<BackupData>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    let mut tx = state.db.begin().await.map_err(AppError::Sqlx)?;

    sqlx::query("DELETE FROM allowance_ledger")
        .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
    sqlx::query("DELETE FROM calendars")
        .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
    sqlx::query("DELETE FROM settings")
        .execute(&mut *tx).await.map_err(AppError::Sqlx)?;
    // Don't delete self
    sqlx::query("DELETE FROM users WHERE id != $1")
        .bind(auth.user_id)
        .execute(&mut *tx).await.map_err(AppError::Sqlx)?;

    let mut user_id_map = std::collections::HashMap::new();

    // Map the old admin/owner to the current one
    if let Some(old_admin) = backup.users.iter().find(|u| matches!(u.role, UserRole::Admin)) {
        user_id_map.insert(old_admin.id, auth.user_id);
    }

    for user in backup.users {
        if user.id == auth.user_id || user_id_map.contains_key(&user.id) {
            continue;
        }
        
        let new_id = uuid::Uuid::new_v4();
        user_id_map.insert(user.id, new_id);

        sqlx::query(
            "INSERT INTO users (id, username, name, password_hash, birthday, profile_picture_url, role, track_allowance, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(new_id)
        .bind(user.username)
        .bind(user.name)
        .bind(user.password_hash)
        .bind(user.birthday)
        .bind(user.profile_picture_url)
        .bind(user.role.to_string())
        .bind(user.track_allowance)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Sqlx)?;
    }

    for setting in backup.settings {
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES ($1, $2, $3)
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = EXCLUDED.updated_at"
        )
        .bind(setting.key)
        .bind(setting.value)
        .bind(setting.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Sqlx)?;
    }

    for calendar in backup.calendars {
        sqlx::query(
            "INSERT INTO calendars (id, name, url, google_id, color, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(calendar.name)
        .bind(calendar.url)
        .bind(calendar.google_id)
        .bind(calendar.color)
        .bind(calendar.created_at)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Sqlx)?;
    }

    for entry in backup.allowance_ledger {
        if let Some(new_user_id) = user_id_map.get(&entry.user_id) {
            sqlx::query(
                "INSERT INTO allowance_ledger (id, user_id, amount, balance, description, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(uuid::Uuid::new_v4())
            .bind(new_user_id)
            .bind(entry.amount)
            .bind(entry.balance)
            .bind(entry.description)
            .bind(entry.created_at)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Sqlx)?;
        }
    }

    tx.commit().await.map_err(AppError::Sqlx)?;

    Ok(StatusCode::OK)
}