use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::user::{AllowanceTransaction, CreateTransactionSchema, UserBalance},
    state::AppState,
    utils::auth_helpers::require_admin,
    middleware::auth::AuthUser,
};

pub async fn add_transaction(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: AuthUser,
    Json(payload): Json<CreateTransactionSchema>,
) -> Result<Json<AllowanceTransaction>, AppError> {
    require_admin(&auth)?;

    // Verify user exists
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    if !user_exists {
        return Err(AppError::UserNotFound);
    }

    if payload.description.len() > 500 {
        return Err(AppError::InvalidInput("Description too long".to_string()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::Sqlx)?;

    let latest_balance: Option<i64> = sqlx::query_scalar(
        "SELECT balance FROM allowance_ledger WHERE user_id = $1 ORDER BY seq DESC LIMIT 1"
    )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await.map_err(AppError::Sqlx)?;

    let current_balance = latest_balance.unwrap_or(0);
    let new_balance = current_balance + payload.amount;
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO allowance_ledger (id, user_id, amount, balance, description)
        VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(id)
    .bind(user_id)
    .bind(payload.amount)
    .bind(new_balance)
    .bind(payload.description)
    .execute(&mut *tx)
    .await.map_err(AppError::Sqlx)?;

    let transaction = query_as::<_, AllowanceTransaction>(
        "SELECT * FROM allowance_ledger WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await.map_err(AppError::Sqlx)?;

    tx.commit().await.map_err(AppError::Sqlx)?;

    Ok(Json(transaction))
}

pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Vec<AllowanceTransaction>>, AppError> {
    // Users can view their own ledger, admins can view anyone
    if !auth.is_admin() && auth.user_id != user_id {
        return Err(AppError::AuthError);
    }

    // Verify user exists
    let user_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    if !user_exists {
        return Err(AppError::UserNotFound);
    }

    let ledger = query_as::<_, AllowanceTransaction>(
        "SELECT * FROM allowance_ledger WHERE user_id = $1 ORDER BY seq DESC"
    )
        .bind(user_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ledger))
}

pub async fn get_balances(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<UserBalance>>, AppError> {
    let balances = if auth.is_admin() {
        query_as::<_, UserBalance>(
            r#"
            SELECT u.id as user_id, u.name, COALESCE(
                (SELECT balance FROM allowance_ledger 
                 WHERE user_id = u.id 
                 ORDER BY seq DESC 
                 LIMIT 1), 0) as balance
            FROM users u
            WHERE u.track_allowance = 1
            ORDER BY u.name
            "#
        )
        .fetch_all(&state.db)
        .await?
    } else {
        query_as::<_, UserBalance>(
            r#"
            SELECT u.id as user_id, u.name, COALESCE(
                (SELECT balance FROM allowance_ledger 
                 WHERE user_id = u.id 
                 ORDER BY seq DESC 
                 LIMIT 1), 0) as balance
            FROM users u
            WHERE u.id = $1 AND u.track_allowance = 1
            "#
        )
        .bind(auth.user_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(balances))
}