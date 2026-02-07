use serde::{Deserialize, Serialize};
use crate::models::{
    user::{BackupUser, AllowanceTransaction},
    settings::Setting,
    calendar::Calendar,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub users: Vec<BackupUser>,
    pub settings: Vec<Setting>,
    pub calendars: Vec<Calendar>,
    pub allowance_ledger: Vec<AllowanceTransaction>,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
