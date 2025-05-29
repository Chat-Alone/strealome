use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub id:         i32,
    pub author_id:  i32,
    pub room:       String,
    pub content:    MessagePayload,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum MessagePayload {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "meme")]
    Meme(String),
    #[serde(rename = "file")]
    File {
        name: String,
        raw: Vec<u8>
    },
}