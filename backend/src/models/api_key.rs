use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    #[serde(skip)]
    pub key_hash: String,
    pub name: String,
    pub user_id: Uuid,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeySchema {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreatedApiKeyResponse {
    pub id: Uuid,
    pub key: String, // Only returned once on creation
    pub name: String,
}
