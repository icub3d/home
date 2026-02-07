use axum::{
    extract::State,
    http::{Request, header},
    middleware::Next,
    response::IntoResponse,
    body::Body,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    state::AppState,
    utils::jwt::{verify_jwt, create_jwt, should_refresh_token},
    models::user::UserRole,
};

pub async fn sliding_session_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    
    let mut new_token = None;

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let jwt_secret = state.jwt_secret.read().await;
                
                if let Ok(claims) = verify_jwt(token, jwt_secret.as_bytes()) {
                    if should_refresh_token(&claims) {
                        // Refresh the token
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            if let Ok(role) = claims.role.parse::<UserRole>() {
                                if let Ok(refreshed) = create_jwt(user_id, &role, jwt_secret.as_bytes()) {
                                    new_token = Some(refreshed);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut response = next.run(req).await;

    if let Some(token) = new_token {
        if let Ok(value) = header::HeaderValue::from_str(&token) {
            response.headers_mut().insert("x-new-token", value);
            // Ensure the header is exposed to JS
            response.headers_mut().append(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                header::HeaderValue::from_static("x-new-token")
            );
        }
    }

    response
}