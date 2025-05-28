use axum::{routing, Router, response::Response as AxumResponse};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;

use super::{Response, AppState, Jwt};
use crate::service::{chat, room};

async fn upgrade(
    jwt: Jwt, State(state): State<AppState>,
    ws: WebSocketUpgrade, Path(room_link): Path<String>
) -> AxumResponse {
    if let Err(e) = room::get_room_by_link(&room_link) {
        return Response::from(e).into_response();
    }

    ws.on_upgrade(
        async move |socket| {
            if let Err(e) = 
                chat::handle_websocket(socket, room_link, jwt.sub, state.repository).await {
                eprintln!("Error: {}", e)
            }
        }
    )
}

pub fn route(path: &str) -> Router<AppState> {
    let path = if path == "/" { "/{room}" } else { &format!("{path}/{{room}}") };
    Router::new()
        .route(path, routing::get(upgrade))
}
