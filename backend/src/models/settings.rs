use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub family_name: String,
    pub base_url: String,
    pub weather_zip_code: String,
    pub background_url: String,
    pub openweather_api_key: String,
    pub google_client_id: String,
    pub google_client_secret: String,

    // Indicate if Google account is connected (has refresh token)
    pub google_connected: bool,

    #[serde(skip_serializing)]
    pub google_photos_access_token: String,
    #[serde(skip_serializing)]
    pub google_photos_refresh_token: String,
    #[serde(skip_serializing)]
    pub google_photos_token_expiry: String,
    #[serde(skip_serializing)]
    pub google_photos_album_id: String,
    #[serde(skip_serializing)]
    pub google_photos_picked_items: String,

    #[serde(skip_serializing)]
    pub last_background_refresh: String,
}

#[derive(Debug, Deserialize)]

pub struct UpdateAppSettingsSchema {

    pub family_name: Option<String>,

    pub base_url: Option<String>,

    pub weather_zip_code: Option<String>,

    pub background_url: Option<String>,

    pub openweather_api_key: Option<String>,

    pub google_client_id: Option<String>,

    pub google_client_secret: Option<String>,

}
