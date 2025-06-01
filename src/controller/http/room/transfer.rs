use axum::extract::{ Json };
use axum::{ routing, Router };
use crate::service::room;
use super::{AppState, Response, Jwt};


#[derive(serde::Deserialize)]
struct PostRequest {
    room:       String,
    target_id:  i32,
}

async fn post(jwt: Jwt, Json(req): Json<PostRequest>) -> Response {
    let old_host_id = jwt.sub;
    if let Err(e) = room::contains_user(&req.room, old_host_id) { return e.into() };
    
    let PostRequest { room: target_room, target_id: new_host_id } = req;
    if let Err(e) = room::change_host(&target_room, new_host_id).await { return e.into() };
    
    Response::success::<()>(None)
}

pub fn route(path: &str) -> Router< AppState> {
    Router::new()
        .route(path, routing::post(post))
}