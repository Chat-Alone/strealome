use chrono::{DateTime, Utc};
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl UserModel {
    pub fn new_user(name: String, password: String) -> Self {
        Self {
            id: 0,
            name,
            password,
            created_at: Utc::now(),
        }
    }
}
