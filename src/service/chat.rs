use thiserror::Error as ThisError;

use std::sync::Arc;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use super::Repository;

#[derive(ThisError, Debug)]
pub enum ChatError {
    #[error("User not found.")]
    UserNotFound,
    #[error("Service error")]
    ServiceError(#[from] super::Error),
}

pub async fn handle_websocket(
    mut socket: WebSocket,
    user_id: i32, repo: Arc<dyn Repository>
) -> Result<(), ChatError> {
    let user = repo.find_by_id(user_id).await;
    if user.is_none() {
        return Err(ChatError::UserNotFound);
    };
    
    let user = user.unwrap();
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                let msg = text.as_bytes();
                let msg = std::str::from_utf8(msg).unwrap();
                let msg = msg.to_string();
                socket.send(Message::Text(msg.into())).await.unwrap();
            },
            Message::Binary(_) => {}
            _ => {}
        }
    }
        
    Ok(())
}