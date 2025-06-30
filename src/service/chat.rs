use thiserror::Error as ThisError;

use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::controller::Response;
use super::{room, user, Repository};
use crate::model::chat::{Signal as ChatSignal, Event as ChatEvent, Payload as ChatPayload};
use crate::service::room::Rooms;

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

impl From<&ChatSignal> for Message {
    fn from(value: &ChatSignal) -> Self {
        Message::Text(serde_json::to_string(value).unwrap().into())
    }
}

pub async fn handle_websocket(
    socket: WebSocket, room_link: String,
    user_id: i32, repo: Arc<dyn Repository>, rooms: Rooms
) -> Result<(), ChatError> {
    let user = user::get_user_by_id(repo, user_id).await?;
    let (tx, mut rx) = mpsc::channel::<Arc<ChatSignal>>(MPSC_BUF_SIZE);
    let (mut sender, mut recver) = socket.split();
    
    rooms.join_room(&room_link, user_id, tx.clone()).await?;
    
    println!("CurrRooms: {:?}", rooms);
    
    let _tx = tx.clone();
    let recv_fut = async move {
        while let Some(Ok(msg)) = recver.next().await {
            println!("recv: {:?}", msg);
            match msg {
                Message::Text(text) => {
                    if let Ok(signal) = serde_json::from_str::<ChatSignal>(&text) {
                        if let Some(ret) = ping_pong(&signal) {
                            if let Err(e) = _tx.send(Arc::new(ret)).await {
                                eprintln!("send error: {:?}", e)
                            }
                        }
                    }
                },
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    };
    
    let send_fut = async move {
        while let Some(msg) = rx.recv().await {
            println!("sending: {:?}", msg);
            sender.send(msg.as_ref().into()).await.map_err(ChatError::from)?;
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
    
    let _ = rooms.leave_room(&room_link, user_id).await;
        
    Ok(())
}

// use super::room::{Room};

// pub async fn create_room(repo: Arc<dyn Repository>, user_id: i32) -> Result<Room, ChatError> {
//     let user = user::get_user_by_id(repo, user_id).await?;
//     Ok(room::create_host_by(user.id))
// }

pub async fn send_message(
    repo: Arc<dyn Repository>, rooms: &Rooms,
    user_id: i32, room_link: String, content: String
) -> Result<(), ChatError> {
    let user = user::get_user_by_id(repo, user_id).await?;
    rooms.send_message(user.id, room_link, content).await?;
    Ok(())
}

fn ping_pong(signal: &ChatSignal) -> Option<ChatSignal> {
    match signal.payload() {
        ChatPayload::Ping(ping) => Some(ChatPayload::pong(ping)),
        _ => None,
    }.map(|payload| ChatSignal::sys(1919810, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::chat::{Signal as ChatSignal, Event as ChatEvent, Payload as ChatPayload};
    
    #[test]
    fn test_ping_pong() {
        let signal = ChatSignal::new(1919810, 10, ChatPayload::ping(114514, 1919810));
        if let ChatPayload::Pong(pong) = ping_pong(&signal).expect("Invalid signal").payload() {
            assert_eq!(pong.id, 114514);
            assert_eq!(pong.sn, 1919810);
        } else {
            panic!("Invalid pong")
        }
    }
}
