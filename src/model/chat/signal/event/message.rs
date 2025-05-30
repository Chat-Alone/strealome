use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub id:         i32,
    pub author_id:  i32,
    #[serde(flatten)]
    pub content:    MessagePayload,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn text(id: i32, author_id: i32, text: String) -> Self {
        Self {
            id,
            author_id,
            content:    MessagePayload::Text(text),
            created_at: Utc::now(),
        }
    }
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