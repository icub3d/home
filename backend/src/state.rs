use sqlx::SqlitePool;
use std::sync::Arc;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CachedPhotos {
    pub source_url: String,
    pub images: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: Arc<RwLock<String>>,
    pub photo_cache: Arc<RwLock<Option<CachedPhotos>>>,
    pub http_client: reqwest::Client,
    pub openweather_api_key: Arc<RwLock<String>>,
    pub google_client_id: Arc<RwLock<String>>,
    pub google_client_secret: Arc<RwLock<String>>,
    pub google_oauth_redirect_uri: Arc<RwLock<String>>,
    pub base_url: Arc<RwLock<String>>,
    pub photos_dir: PathBuf,
}
