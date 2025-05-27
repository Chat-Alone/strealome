use axum::extract::{State, Json};
use serde::Deserialize;
use crate::model::ChatMessageContent;
use super::{AppState, Jwt, Response};
use crate::service::{chat, room, user};

#[derive(Debug, Deserialize)]
struct PostRequest {
    room:       String,
    content:    String,
}

async fn post(jwt: Jwt, State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let res = chat::send_message(state.repository, jwt.sub, &req.room, req.content).await;
    match res {
        Ok(_) => Response::success::<()>(None),
        Err(e) => Response::from(e),
    }
}

pub async fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::post(post))
}