use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chat {
    pub id:         i32,
    pub author:     String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ChatPayload {
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