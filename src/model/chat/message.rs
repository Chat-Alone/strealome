use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;

use super::gen_id;

#[derive(Debug)]
pub struct Message {
    id:         i32,
    author_id:  i32,
    room:       String,
    content:    RwLock<MessagePayload>,
    created_at: DateTime<Utc>,

    formatted:  Arc<RwLock<String>>,
}

impl Message {
    pub fn new(author_id: i32, room: String, content: MessagePayload) -> Self {
        let id = gen_id();
        let created_at = Utc::now();
        let formatted = json!({
            "id": id,
            "author_id": author_id,
            "room": room,
            "content": content,
            "created_at": created_at.to_rfc3339(),
        }).to_string();

        Self {
            id,
            room,
            author_id,
            created_at,
            content: RwLock::new(content),
            formatted: Arc::new(RwLock::new(formatted)),
        }
    }

    pub fn joined(author_id: i32, room: String) {
        todo!()
    }
    pub fn author(&self) -> i32 {
        self.author_id
    }

    pub async fn serialize(&self) -> String {
        self.formatted.read().await.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
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