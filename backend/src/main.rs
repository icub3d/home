mod error;
mod handlers;
mod middleware;
mod models;
mod state;
mod utils;
mod background;

use axum::{
    routing::{get, post, put, delete},
    middleware as axum_middleware,
    Router,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer, key_extractor::SmartIpKeyExtractor};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;
use crate::handlers::{auth, user, allowance, settings, calendar, backup, display, chore, weather, google_photos};

fn env_bool(key: &str) -> bool {
    matches!(
        std::env::var(key).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

async fn get_setting(pool: &sqlx::SqlitePool, key: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .unwrap_or_default()
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "backend=info,tower_http=info,axum=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/home.db".to_string());
    
    // Create database connection pool
    let connection_options = database_url.parse::<sqlx::sqlite::SqliteConnectOptions>()
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true);

    // Ensure parent directory exists for SQLite
    let path = connection_options.get_filename();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("Failed to create database directory");
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options.clone())
        .await
        .expect("Failed to connect to the database");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Load settings from DB or Env
    let mut jwt_secret = get_setting(&pool, "jwt_secret").await;
    if jwt_secret.is_empty() {
        tracing::info!("Generating new JWT secret");
        jwt_secret = utils::auth_helpers::generate_random_token(64);
        sqlx::query("INSERT INTO settings (key, value) VALUES ('jwt_secret', $1)")
            .bind(&jwt_secret)
            .execute(&pool)
            .await
            .expect("Failed to save JWT secret to database");
    }

    let openweather_api_key = if let Ok(s) = std::env::var("OPENWEATHER_API_KEY") { s } else { get_setting(&pool, "openweather_api_key").await };
    let google_client_id = if let Ok(s) = std::env::var("GOOGLE_CLIENT_ID") { s } else { get_setting(&pool, "google_client_id").await };
    let google_client_secret = if let Ok(s) = std::env::var("GOOGLE_CLIENT_SECRET") { s } else { get_setting(&pool, "google_client_secret").await };
    
    let google_oauth_redirect_uri = if let Ok(s) = std::env::var("GOOGLE_OAUTH_REDIRECT_URI") { s } else {
        let val = get_setting(&pool, "google_oauth_redirect_uri").await;
        if val.is_empty() { "http://localhost:4000/api/google-photos/callback".to_string() } else { val }
    };

    let base_url = if let Ok(s) = std::env::var("BASE_URL") { s } 
        else if let Ok(s) = std::env::var("CORS_ORIGIN") { s } 
        else {
            let val = get_setting(&pool, "base_url").await;
            if val.is_empty() { "http://localhost:4000".to_string() } else { val }
        };

    let photos_dir = if let Ok(s) = std::env::var("PHOTOS_DIR") {
        PathBuf::from(s)
    } else {
        PathBuf::from("data/photos")
    };

    // Ensure photos directory exists
    if !photos_dir.as_os_str().is_empty() {
        std::fs::create_dir_all(&photos_dir).expect("Failed to create photos directory");
    }

    let state = Arc::new(AppState { 
        db: pool,
        jwt_secret: Arc::new(RwLock::new(jwt_secret)),
        photo_cache: Arc::new(RwLock::new(None)),
        http_client: reqwest::Client::new(),
        openweather_api_key: Arc::new(RwLock::new(openweather_api_key)),
        google_client_id: Arc::new(RwLock::new(google_client_id)),
        google_client_secret: Arc::new(RwLock::new(google_client_secret)),
        google_oauth_redirect_uri: Arc::new(RwLock::new(google_oauth_redirect_uri)),
        base_url: Arc::new(RwLock::new(base_url)),
        photos_dir,
    });

    // Configure rate limiting for auth endpoints
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap()
    );

    let auth_rate_limit = GovernorLayer::new(governor_conf.clone());
    let register_rate_limit = GovernorLayer::new(governor_conf.clone());

    let api = Router::new()
        // Auth routes
        .route("/auth/status", get(auth::get_status))
        .route("/auth/login", post(auth::login).layer(auth_rate_limit))
        .route("/auth/register", post(auth::register).layer(register_rate_limit))
        // User routes
        .route("/users", get(user::get_users).post(user::create_user))
        .route("/users/{id}", get(user::get_user).put(user::update_user).delete(user::delete_user))
        .route("/users/{id}/password", put(user::change_password))
        // Allowance routes
        .route("/allowance/balances", get(allowance::get_balances))
        .route("/allowance/{user_id}", get(allowance::get_ledger))
        .route("/allowance/{user_id}/transaction", post(allowance::add_transaction))
        // Settings routes
        .route("/settings", get(settings::get_settings).put(settings::update_settings))
        // Calendar routes
        .route("/calendars", get(calendar::list_calendars).post(calendar::create_calendar))
        .route("/calendars/google", get(calendar::list_google_calendars))
        .route("/calendars/{id}", delete(calendar::delete_calendar))
        .route("/calendars/{id}/feed", get(calendar::get_calendar_feed))
        // Backup routes
        .route("/backup/export", get(backup::export_backup))
        .route("/backup/import", post(backup::import_backup))
        // Weather route
        .route("/weather", get(weather::get_weather))
        // Display routes
        .route("/display/tokens", get(display::list_tokens).post(display::create_token))
        .route("/display/tokens/{id}", delete(display::delete_token))
        .route("/display/data", get(display::get_display_data))
        // Google Photos routes
        .route("/google-photos/start", post(google_photos::start_google_oauth))
        .route("/google-photos/callback", get(google_photos::google_oauth_callback))
        .route("/google-photos/session", post(google_photos::create_session))
        .route("/google-photos/confirm", post(google_photos::confirm_selection))
        .route("/google-photos/disconnect", post(google_photos::disconnect_google_photos))
        // Chore routes
        .route("/chores", get(chore::list_chores).post(chore::create_chore))
        .route("/chores/{id}", put(chore::update_chore).delete(chore::delete_chore))
        .route("/chores/{id}/toggle", put(chore::toggle_complete))
        // Static files
        .route("/photos/{filename}", get(google_photos::get_photo))
        .layer(axum_middleware::from_fn_with_state(state.clone(), crate::middleware::session::sliding_session_middleware));

    let serve_frontend = env_bool("SERVE_FRONTEND");

    let app = if serve_frontend {
        let frontend_dir = std::env::var("FRONTEND_DIST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("public")))
                    .unwrap_or_else(|| PathBuf::from("public"))
            });
        let index = frontend_dir.join("index.html");

        Router::new()
            .route("/health", get(health_check))
            .nest("/api", api)
            .fallback_service(ServeDir::new(frontend_dir).not_found_service(ServeFile::new(index)))
    } else {
        Router::new()
            .route("/health", get(health_check))
            .nest("/api", api)
    }
    .layer(SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    ))
    .with_state(state.clone());

    // Run it
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(4001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    background::start_refresh_loop(state.clone());

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn health_check() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}
