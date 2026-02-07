use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Calendar {
    pub id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub google_id: Option<String>,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CalendarPublic {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCalendarSchema {
    pub name: String,
    pub url: Option<String>,
    pub google_id: Option<String>,
    pub color: Option<String>,
}
