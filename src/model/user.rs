use chrono::{DateTime, Utc};
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone)]
pub struct UserModel {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}
