use axum::{
    routing::get,
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use futures::{SinkExt, StreamExt};

type BroadcastChannel = broadcast::Sender<String>;

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let (tx, _) = broadcast::channel(100);
    
    let (mut sender, mut receiver) = socket.split();

    let _tx = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.into_text() {
                let _ = _tx.send(text);
            }
        }
    });
    
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = tx.subscribe().recv().await {
            if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }
}

pub fn signal_router() -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
}