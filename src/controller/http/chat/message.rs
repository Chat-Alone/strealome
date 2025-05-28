use axum::{routing, Router};
use axum::extract::{State, Json};
use serde::Deserialize;
use super::{AppState, Jwt, Response};
use crate::service::{chat};

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

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::post(post))
}