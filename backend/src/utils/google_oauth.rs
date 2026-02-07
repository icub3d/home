use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc, Duration};

use crate::state::AppState;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_PICKER_API: &str = "https://photospicker.googleapis.com/v1";
const GOOGLE_CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickerSession {
    pub id: String,
    #[serde(rename = "pickerUri")]
    pub picker_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickerMediaItem {
    pub id: String,
    #[serde(rename = "mediaFile")]
    pub media_file: PickerMediaFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickerMediaFile {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickerMediaItemsResponse {
    #[serde(rename = "mediaItems")]
    pub media_items: Option<Vec<PickerMediaItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCalendarList {
    pub items: Vec<GoogleCalendarListEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCalendarListEntry {
    pub id: String,
    pub summary: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleEventList {
    pub items: Vec<GoogleEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleEvent {
    pub id: String,
    pub summary: Option<String>,
    pub start: Option<GoogleDateTime>,
    pub end: Option<GoogleDateTime>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>, // RFC3339
    pub date: Option<String>, // YYYY-MM-DD
}

pub fn build_auth_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&state={}&prompt=consent",
        GOOGLE_AUTH_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode("https://www.googleapis.com/auth/photospicker.mediaitems.readonly https://www.googleapis.com/auth/calendar.readonly profile email"),
        urlencoding::encode(state)
    )
}

pub async fn exchange_code_for_tokens(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    
    let token_response = client
        .post(GOOGLE_TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={}&client_secret={}&code={}&redirect_uri={}&grant_type=authorization_code",
            urlencoding::encode(client_id),
            urlencoding::encode(client_secret),
            urlencoding::encode(code),
            urlencoding::encode(redirect_uri)
        ))
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(token_response)
}

pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    
    let token_response = client
        .post(GOOGLE_TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
            urlencoding::encode(client_id),
            urlencoding::encode(client_secret),
            urlencoding::encode(refresh_token)
        ))
        .send()
        .await?;
    
    let status = token_response.status();
    if !status.is_success() {
        let body = token_response.text().await?;
        return Err(format!("Google token refresh failed: {} - {}", status, body).into());
    }

    let tokens = token_response.json::<TokenResponse>().await?;
    Ok(tokens)
}

pub async fn get_google_credentials(state: &AppState) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = state.google_client_id.read().await;
    let client_secret = state.google_client_secret.read().await;
    
    if client_id.is_empty() || client_secret.is_empty() {
        return Err("Google OAuth not configured.".into());
    }
    Ok((client_id.clone(), client_secret.clone()))
}

pub async fn get_valid_access_token(db: &SqlitePool, state: &AppState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let access_token: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'google_photos_access_token'"
    )
        .fetch_optional(db)
        .await?;
    
    let token_expiry: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'google_photos_token_expiry'"
    )
        .fetch_optional(db)
        .await?;

    let access_token = access_token.ok_or("No access token found")?;
    
    if let Some(expiry_str) = token_expiry {
        if let Ok(expiry) = expiry_str.parse::<DateTime<Utc>>() {
            if Utc::now() >= expiry {
                let refresh_token: Option<String> = sqlx::query_scalar(
                    "SELECT value FROM settings WHERE key = 'google_photos_refresh_token'"
                )
                    .fetch_optional(db)
                    .await?;
                
                let refresh_token = refresh_token.ok_or("No refresh token found")?;
                let (client_id, client_secret) = get_google_credentials(state).await?;

                let token_response = refresh_access_token(&client_id, &client_secret, &refresh_token).await?;
                let new_expiry = Utc::now() + Duration::seconds(token_response.expires_in);
                
                sqlx::query(
                    "INSERT INTO settings (key, value) VALUES ('google_photos_access_token', $1) 
                     ON CONFLICT (key) DO UPDATE SET value = $1"
                )
                    .bind(&token_response.access_token)
                    .execute(db)
                    .await?;
                
                sqlx::query(
                    "INSERT INTO settings (key, value) VALUES ('google_photos_token_expiry', $1) 
                     ON CONFLICT (key) DO UPDATE SET value = $1"
                )
                    .bind(new_expiry.to_rfc3339())
                    .execute(db)
                    .await?;

                return Ok(token_response.access_token);
            }
        }
    }

    Ok(access_token)
}

pub async fn create_picker_session(access_token: &str) -> Result<PickerSession, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let response = client
        .post(format!("{}/sessions", GOOGLE_PICKER_API))
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .body("{}") 
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(format!("Google Picker API error: {} - {}", status, body).into());
    }

    let session: PickerSession = response.json().await?;
    Ok(session)
}

pub async fn list_picked_media_items(
    access_token: &str,
    session_id: &str,
) -> Result<Vec<PickerMediaItem>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let url = format!("{}/mediaItems?sessionId={}", GOOGLE_PICKER_API, urlencoding::encode(session_id));

    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(format!("Google Picker API error: {} - {}", status, body).into());
    }

    let body = response.text().await?;
    let response_data: PickerMediaItemsResponse = serde_json::from_str(&body)?;
    Ok(response_data.media_items.unwrap_or_default())
}

pub async fn list_calendars(access_token: &str) -> Result<Vec<GoogleCalendarListEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let response = client
        .get(format!("{}/users/me/calendarList", GOOGLE_CALENDAR_API))
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(format!("Google Calendar API error: {} - {}", status, body).into());
    }

    let list: GoogleCalendarList = response.json().await?;
    Ok(list.items)
}

pub async fn get_calendar_events(access_token: &str, calendar_id: &str) -> Result<Vec<GoogleEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let now = Utc::now().to_rfc3339();
    
    let response = client
        .get(format!("{}/calendars/{}/events?timeMin={}&singleEvents=true&orderBy=startTime&maxResults=50", 
            GOOGLE_CALENDAR_API, 
            urlencoding::encode(calendar_id),
            urlencoding::encode(&now)
        ))
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(format!("Google Calendar API error: {} - {}", status, body).into());
    }

    let list: GoogleEventList = response.json().await?;
    Ok(list.items)
}
