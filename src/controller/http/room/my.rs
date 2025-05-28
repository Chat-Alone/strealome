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
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let related_rooms = room::related_to(user.id)
            .into_iter().map(|r| RoomResp::from(r, user.id)).collect();
    
    Response::success(Some(GetResponse { related_rooms }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::get(get))
}