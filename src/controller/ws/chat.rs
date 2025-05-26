use axum::{routing, Router, response::Response as AxumResponse};
use axum::extract::{State, WebSocketUpgrade};
use super::{AppState, Jwt};
use crate::service::chat;

async fn upgrade(jwt: Jwt, State(state): State<AppState>, ws: WebSocketUpgrade) -> AxumResponse {
    ws.on_upgrade(
        async move |socket| {
            if let Err(e) = chat::handle_websocket(socket, jwt.sub, state.repository).await {
                eprintln!("Error: {}", e);
            }
        }
    )
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::get(upgrade))
}