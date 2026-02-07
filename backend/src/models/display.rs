use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::{user::UserBalance, calendar::CalendarPublic, chore::ChoreWithUser};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DisplayToken {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenSchema {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DisplayData {
    pub weather: Option<serde_json::Value>,
    pub calendars: Vec<CalendarPublic>,
    pub allowances: Vec<UserBalance>,
    pub chores: Vec<ChoreWithUser>,
    pub background_url: Option<String>,
}