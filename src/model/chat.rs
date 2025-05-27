use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

static ID: AtomicI32  = AtomicI32::new(0);
pub fn gen_id() -> i32 {
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug)]
pub struct ChatMessage {
    id:         i32,
    author_id:  i32,
    room:       String,
    content:    RwLock<ChatMessageContent>,
    created_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(author_id: i32, room: String, content: ChatMessageContent) -> Self {
        Self {
            id: gen_id(),
            author_id,
            room,
            content: RwLock::new(content),
            created_at: Utc::now(),
        }
    }
    
    pub fn author(&self) -> i32 {
        self.author_id
    }
    
}

#[derive(Debug)]
pub enum ChatMessageContent {
    Text(String),
    Meme(String),
    File(String, Vec<u8>),
}