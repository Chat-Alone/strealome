use thiserror::Error as ThisError;

use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::controller::Response;
use super::{room, user, Repository};
use crate::model::{ChatMessage, ChatMessageContent};

const MPSC_BUF_SIZE: usize = 32;

#[derive(ThisError, Debug)]
pub enum ChatError {
    #[error("{0}")]
    UserError(#[from] user::UserError),

    #[error("{0}")]
    RoomError(#[from] room::RoomError),
    
    #[error("Socket error")]
    SocketError(#[from] axum::Error),
    
    #[error("Internal error")]
    InternalError,
    
    #[error("Service error")]
    ServiceError(#[from] super::Error),
}

impl From<ChatError> for Response {
    fn from(e: ChatError) -> Self {
        Response::error(&e.to_string())
    }
}

pub async fn handle_websocket(
    socket: WebSocket, room_link: String,
    user_id: i32, repo: Arc<dyn Repository>
) -> Result<(), ChatError> {
    let user = user::get_user_by_id(repo, user_id).await?;
    let (tx, mut rx) = mpsc::channel::<Arc<ChatMessage>>(MPSC_BUF_SIZE);
    let (mut sender, mut recver) = socket.split();

    let room = room::get_room_by_link(&room_link)?;
    room.join(user_id, tx.clone()).await;
    
    println!("CurrRooms: {:?}", room::rooms());
    
    let _tx = tx.clone();
    let recv_fut = async move {
        while let Some(Ok(msg)) = recver.next().await {
            println!("recv: {:?}", msg);
            if let Message::Text(text) = msg {
                if let Ok(content) = serde_json::from_str::<ChatMessageContent>(&text) {
                    room.sync_message(user.id, content).await?;
                } else {
                    // TODO! 
                    println!("bad message: {}", text);
                    _tx.send(Arc::new(ChatMessage::new(user.id, room_link.clone(), 
                        ChatMessageContent::Text("发的不对你这个".to_string())))
                    ).await.map_err(|_| ChatError::InternalError)?;
                }
            }
        }
        Ok(())
    };
    
    let send_fut = async move {
        while let Some(msg) = rx.recv().await {
            println!("sending: {:?}", msg);
            sender.send(msg.serialize().await.into()).await.map_err(|e| ChatError::from(e))?;
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

use super::room::{Room};

pub async fn create_room(repo: Arc<dyn Repository>, user_id: i32) -> Result<Room, ChatError> {
    let user = user::get_user_by_id(repo, user_id).await?;
    Ok(room::create_host_by(user.id))
}

pub async fn send_message(repo: Arc<dyn Repository>, user_id: i32, room_link: &str, content: String) -> Result<(), ChatError> {
    let user = user::get_user_by_id(repo, user_id).await?;
    let room = room::get_room_by_link(room_link)?;
    room.sync_message(user.id, ChatMessageContent::Text(content)).await?;
    Ok(())
}

impl From<&ChatMessage> for Message {
    fn from(msg: &ChatMessage) -> Self {
        Message::Text(format!("{:?}", &msg).into())
    }
}