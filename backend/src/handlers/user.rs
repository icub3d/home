use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::user::{User, CreateUserSchema, UpdateUserSchema, UserRole, ChangePasswordSchema},
    state::AppState,
    utils::{jwt::hash_password, auth_helpers::require_admin},
    middleware::auth::AuthUser,
};

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<User>>, AppError> {
    require_admin(&auth)?;

    let users = query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<User>, AppError> {
    // Users can view their own profile, admins can view anyone
    if !auth.is_admin() && auth.user_id != id {
        return Err(AppError::AuthError);
    }

    let user = query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateUserSchema>,
) -> Result<Json<User>, AppError> {
    require_admin(&auth)?;

    if payload.password.len() < 12 {
        return Err(AppError::InvalidInput("Password must be at least 12 characters".to_string()));
    }

    let user_role = payload.role.unwrap_or(UserRole::Member);
    let password_hash = hash_password(&payload.password)?;
    let track_allowance = payload.track_allowance.unwrap_or_default();
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, name, password_hash, birthday, role, track_allowance)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(payload.username)
    .bind(payload.name)
    .bind(password_hash)
    .bind(payload.birthday)
    .bind(user_role.to_string())
    .bind(track_allowance)
    .execute(&state.db)
    .await?;

    let user = query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<UpdateUserSchema>,
) -> Result<Json<User>, AppError> {
    require_admin(&auth)?;

    sqlx::query(
        r#"
        UPDATE users
        SET
            name = COALESCE($1, name),
            birthday = COALESCE($2, birthday),
            role = COALESCE($3, role),
            profile_picture_url = COALESCE($4, profile_picture_url),
            track_allowance = COALESCE($5, track_allowance),
            updated_at = datetime('now')
        WHERE id = $6
        "#,
    )
    .bind(payload.name)
    .bind(payload.birthday)
    .bind(payload.role.map(|r| r.to_string()))
    .bind(payload.profile_picture_url)
    .bind(payload.track_allowance)
    .bind(id)
    .execute(&state.db)
    .await?;

    let user = query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;

    // Prevent deleting self
    if id == auth.user_id {
        return Err(AppError::InvalidInput("Cannot delete yourself".to_string()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::UserNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<ChangePasswordSchema>,
) -> Result<StatusCode, AppError> {
    // Only admin or self can change password
    if !auth.is_admin() && auth.user_id != id {
        return Err(AppError::AuthError);
    }

    if payload.password.len() < 12 {
        return Err(AppError::InvalidInput("Password must be at least 12 characters".to_string()));
    }

    let password_hash = hash_password(&payload.password)?;

    let result = sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = datetime('now') WHERE id = $2"
    )
        .bind(password_hash)
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::UserNotFound);
    }

    Ok(StatusCode::OK)
}