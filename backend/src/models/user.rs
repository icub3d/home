use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,  // Can manage users, settings
    Member, // Standard family member
    Child,  // Limited access
}

impl UserRole {
    /// Check if role has admin-level privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Member => "member".to_string(),
            UserRole::Child => "child".to_string(),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "member" => Ok(UserRole::Member),
            "child" => Ok(UserRole::Child),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub name: String,
    #[serde(skip)]
    pub password_hash: String,
    pub birthday: Option<NaiveDate>,
    pub profile_picture_url: Option<String>,
    pub role: UserRole,
    pub track_allowance: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BackupUser {
    pub id: Uuid,
    pub username: String,
    pub name: String,
    pub password_hash: String,
    pub birthday: Option<NaiveDate>,
    pub profile_picture_url: Option<String>,
    pub role: UserRole,
    pub track_allowance: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUserSchema {
    pub username: String,
    pub name: String,
    pub password: String,
    pub birthday: NaiveDate,
    pub role: Option<UserRole>,
    pub track_allowance: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateUserSchema {
    pub name: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub role: Option<UserRole>,
    pub profile_picture_url: Option<String>,
    pub track_allowance: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserLoginSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangePasswordSchema {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]

pub struct AllowanceTransaction {

    pub seq: i64,

    pub id: Uuid,

    pub user_id: Uuid,

    pub amount: i64,

    pub balance: i64,

    pub description: String,

    pub created_at: chrono::DateTime<chrono::Utc>,

}



#[derive(Debug, Deserialize)]

pub struct CreateTransactionSchema {

    pub amount: i64,

    pub description: String,

}



#[derive(Debug, Serialize, FromRow)]

pub struct UserBalance {

    pub user_id: Uuid,

    pub name: String,

    pub balance: Option<i64>,

}
