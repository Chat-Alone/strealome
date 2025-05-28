use axum::extract::State;
use axum::Router;
use serde::Serialize;
use super::{AppState, Response, Jwt, RoomResp};
use crate::service::{room, user};

#[derive(Serialize)]
struct PostResponse {
    room: RoomResp,
}

async fn post(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();

    let room = RoomResp::from(room::create_host_by(user.id), user.id);
    Response::success(Some(PostResponse { room }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::post(post))
}