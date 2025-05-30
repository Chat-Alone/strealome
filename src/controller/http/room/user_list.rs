// use std::sync::Arc;
// use tokio::sync::Mutex;
use axum::{routing, Router};
use axum::extract::{State, Json};
use serde::{Deserialize, Serialize};
use crate::model::user::UserPub;
use crate::service::{room, user};
use super::{AppState, Response, Jwt};

#[derive(Deserialize)]
struct PostRequest {
    room:   String,
}

#[derive(Serialize)]
struct PostResponse {
    users: Vec<UserPub>
}

/// 全对
async fn post(jwt: Jwt, State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let req_user_id = jwt.sub;
    let PostRequest { room } = req;
    let target_room = room::get_room_by_link(&room);
    if let Err(e) = target_room { return e.into() }

    // 验证用户
    let target_room = target_room.unwrap();
    if let Err(_) = target_room.contains_user(req_user_id) {
        return Response::error("😡🫵PERMISSION DENIED🫵😡")
    }

    let repo = state.repository;

    let mut users = vec![];
    for user_id in target_room.users() {
        let u = user::get_user_by_id(repo.clone(), user_id).await;
        if let Ok(u) = u {
            users.push(u.into());
        }
    }
    let ret = PostResponse { users };

    // let mut join_handles = vec![];
    // let users: Arc<Mutex<Vec<UserPub>>> = Arc::new(Mutex::new(vec![]));
    // for user_id in target_room.users() {
    //     let repo = repo.clone();
    //     let users = users.clone();
    //     join_handles.push(tokio::spawn(async move {
    //         let u = user::get_user_by_id(repo.clone(), user_id).await;
    //         if let Ok(u) = u {
    //             users.lock().await.push(u.into());
    //         }
    //     }));
    // }
    //
    // for handle in join_handles {
    //     handle.await.unwrap();
    // }
    //
    // let ret = PostResponse { users: users.into_inner() };

    Response::success(Some(ret))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::post(post))
}
