use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_jwt(user_id: Uuid, role: &UserRole, secret: &[u8]) -> Result<String, AppError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 60 * 60 * 24 * 30; // 30 days

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &[u8]) -> Result<Claims, AppError> {

    let token_data = decode::<Claims>(

        token,

        &DecodingKey::from_secret(secret),

        &Validation::default(),

    )?;

    Ok(token_data.claims)

}



pub fn should_refresh_token(claims: &Claims) -> bool {

    let now = SystemTime::now()

        .duration_since(UNIX_EPOCH)

        .unwrap()

        .as_secs() as usize;

    

    // Total duration was 30 days. Halfway is 15 days.

    // If remaining time is less than 15 days, refresh it.

    let total_duration = 60 * 60 * 24 * 30;

    let halfway = total_duration / 2;

    

    (claims.exp - now) < halfway

}
