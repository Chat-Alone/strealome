use thiserror::Error as ThisError;

use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use super::Repository;
use crate::model::{ChatMessage};

const MPSC_BUF_SIZE: usize = 32;

#[derive(ThisError, Debug)]
pub enum ChatError {
    #[error("User not found.")]
    UserNotFound,
    
    #[error("Room not found.")]
    RoomNotFound,
    
    #[error("Socket error")]
    SocketError(#[from] axum::Error),
    
    #[error("Service error")]
    ServiceError(#[from] super::Error),
}

pub async fn handle_websocket(
    socket: WebSocket, room_link: String,
    user_id: i32, repo: Arc<dyn Repository>
) -> Result<(), ChatError> {
    let user = repo.find_by_id(user_id).await;
    if user.is_none() {
        return Err(ChatError::UserNotFound);
    };
    
    let user = user.unwrap();
    let (tx, mut rx) = mpsc::channel::<Arc<ChatMessage>>(MPSC_BUF_SIZE);
    let (mut sender, mut recver) = socket.split();
    
    let room = rooms().get_room_by_link(&room_link).map_err(|_| ChatError::RoomNotFound)?;
    room.join(user_id, tx.clone()).await;
    
    let recv_fut = async move {
        while let Some(Ok(msg)) = recver.next().await {
            println!("{msg:?}");
        }
        Ok(())
    };
    
    let send_fut = async move {
        while let Some(msg) = rx.recv().await {
            if msg.author() == user.id { continue; }
            sender.send(msg.as_ref().into()).await.map_err(|e| ChatError::from(e))?;
        }
        Ok(())
    };
    
    let mut send_task: JoinHandle<Result<(), ChatError>> = tokio::spawn(recv_fut);
    let mut recv_task: JoinHandle<Result<(), ChatError>> = tokio::spawn(send_fut);
    
    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }
        
    Ok(())
}

use super::room::{rooms, Room};

pub async fn create_room(repo: Arc<dyn Repository>, user_id: i32) -> Result<Room, ChatError> {
    let user = repo.find_by_id(user_id).await;
    if user.is_none() {
        return Err(ChatError::UserNotFound);
    };
    let user = user.unwrap();
    Ok(rooms().create(user.id))
}

pub async fn send_message(repo: Arc<dyn Repository>, user_id: i32, room_link: &str, content: String) -> Result<(), ChatError> {
    let user = repo.find_by_id(user_id).await;
    if user.is_none() {
        return Err(ChatError::UserNotFound);
    };
    let user = user.unwrap();
    
    Ok(())
}

impl From<&ChatMessage> for Message {
    fn from(msg: &ChatMessage) -> Self {
        Message::Text(format!("{:?}", &msg).into())
    }
}