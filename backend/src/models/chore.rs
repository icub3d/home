use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Chore {
    pub id: Uuid,
    pub description: String,
    pub assigned_to: Option<Uuid>,
    pub reward: Option<i64>,
    pub completed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChoreWithUser {
    pub id: Uuid,
    pub description: String,
    pub assigned_to: Option<Uuid>,
    pub assigned_name: Option<String>,
    pub reward: Option<i64>,
    pub completed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateChoreSchema {
    pub description: String,
    pub assigned_to: Option<Uuid>,
    pub reward: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateChoreSchema {
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub reward: Option<i64>,
    pub completed: Option<bool>,
}
