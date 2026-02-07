use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::chore::{Chore, ChoreWithUser, CreateChoreSchema, UpdateChoreSchema},
    state::AppState,
    utils::auth_helpers::require_admin,
    middleware::auth::AuthUser,
};

pub async fn list_chores(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<ChoreWithUser>>, AppError> {
    let chores = if auth.is_admin() {
        query_as::<_, ChoreWithUser>(
            r#"
            SELECT c.id, c.description, c.assigned_to, u.name as assigned_name, 
                   c.reward, c.completed, c.created_at, c.updated_at
            FROM chores c
            JOIN users u ON c.assigned_to = u.id
            ORDER BY c.completed ASC, c.created_at DESC
            "#
        )
        .fetch_all(&state.db)
        .await?
    } else {
        query_as::<_, ChoreWithUser>(
            r#"
            SELECT c.id, c.description, c.assigned_to, u.name as assigned_name, 
                   c.reward, c.completed, c.created_at, c.updated_at
            FROM chores c
            JOIN users u ON c.assigned_to = u.id
            WHERE c.assigned_to = $1
            ORDER BY c.completed ASC, c.created_at DESC
            "#
        )
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(chores))
}

pub async fn create_chore(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateChoreSchema>,
) -> Result<Json<Chore>, AppError> {
    require_admin(&auth)?;

    // Verify assigned user exists
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
        .bind(payload.assigned_to)
        .fetch_one(&state.db)
        .await?;

    if !user_exists {
        return Err(AppError::InvalidInput("Assigned user not found".to_string()));
    }

    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO chores (id, description, assigned_to, reward)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(&payload.description)
    .bind(payload.assigned_to)
    .bind(payload.reward)
    .execute(&state.db)
    .await?;

    let chore = query_as::<_, Chore>("SELECT * FROM chores WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(chore))
}

pub async fn update_chore(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<UpdateChoreSchema>,
) -> Result<Json<Chore>, AppError> {
    let chore = query_as::<_, Chore>(
        "SELECT * FROM chores WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidInput("Chore not found".to_string()))?;

    if !auth.is_admin() {
        if chore.assigned_to != Some(auth.user_id) {
            return Err(AppError::AuthError);
        }
        if payload.description.is_some() || payload.assigned_to.is_some() || payload.reward.is_some() {
            return Err(AppError::AuthError);
        }
    }

    // If reassigning, verify new user exists
    if let Some(new_assigned_to) = payload.assigned_to {
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
        )
            .bind(new_assigned_to)
            .fetch_one(&state.db)
            .await?;

        if !user_exists {
            return Err(AppError::InvalidInput("Assigned user not found".to_string()));
        }
    }

    sqlx::query(
        r#"
        UPDATE chores
        SET
            description = COALESCE($1, description),
            assigned_to = COALESCE($2, assigned_to),
            reward = COALESCE($3, reward),
            completed = COALESCE($4, completed),
            updated_at = datetime('now')
        WHERE id = $5
        "#,
    )
    .bind(payload.description)
    .bind(payload.assigned_to)
    .bind(payload.reward)
    .bind(payload.completed)
    .bind(id)
    .execute(&state.db)
    .await?;

    let updated_chore = query_as::<_, Chore>("SELECT * FROM chores WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(updated_chore))
}

pub async fn delete_chore(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    let result = sqlx::query("DELETE FROM chores WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::InvalidInput("Chore not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_complete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Chore>, AppError> {
    let chore = query_as::<_, Chore>(
        "SELECT * FROM chores WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::InvalidInput("Chore not found".to_string()))?;

    // Only admin or assigned user can toggle
    if !auth.is_admin() && chore.assigned_to != Some(auth.user_id) {
        return Err(AppError::AuthError);
    }

    sqlx::query(
        r#"
        UPDATE chores
        SET completed = NOT completed, updated_at = datetime('now')
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await?;
    let updated_chore = query_as::<_, Chore>("SELECT * FROM chores WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(updated_chore))
}