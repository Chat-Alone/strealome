use axum::extract::State;
use axum::Router;
use serde::Serialize;

use crate::service::{user, room};
use super::{Jwt, AppState, Response, RoomResp};

#[derive(Serialize)]
struct GetResponse {
    related_rooms: Vec<RoomResp>,
}

// get all related rooms
async fn get(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository.clone(), jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let user_id = user.id;
    let repo = state.repository;
    let tasks = room::related_to(user_id).into_iter()
        .map(|r| {
            let repo = repo.clone();
            async move {
                RoomResp::from(r, user_id, repo).await
            }
        });
    let related_rooms = futures::future::join_all(tasks).await
        .into_iter().filter_map(|r: Result<_, room::RoomError>| r.ok()).collect();
    
    Response::success(Some(GetResponse { related_rooms }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::get(get))
}