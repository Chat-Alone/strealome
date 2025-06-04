use axum::extract::{State, Json};
use axum::{Router, routing};
use serde::{Deserialize, Serialize};
use crate::model::user::UserPub;
use crate::service::{room, user};
use super::{AppState, Response, Jwt};

#[derive(Deserialize)]
struct PostRequest {
    user_id:    i32,
    room:       String,
}

#[derive(Serialize)]
struct PostResponse(UserPub);

async fn post(jwt: Jwt, State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let req_user_id = jwt.sub;
    let PostRequest  { user_id, room } = req;
    let target_room = state.rooms.get_room_by_link(&room);
    if let Err(e) = target_room { return e.into() }
    
    // 验证用户
    let target_room = target_room.unwrap();
    if let Err(_) = target_room.contains_user(req_user_id) {
        return Response::error("😡🫵PERMISSION DENIED🫵😡")
    }
    if let Err(e) = target_room.contains_user(user_id) { return e.into() }
    
    let target_user = user::get_user_by_id(state.repository.clone(), user_id).await;
    if let Err(e) = target_user { return e.into() }
    let target_user = PostResponse(target_user.unwrap().into());
    
    Response::success(Some(target_user))
    
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::post(post))
}