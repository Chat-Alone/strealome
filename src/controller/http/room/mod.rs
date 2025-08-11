mod my;
mod create;
mod response;
mod detail;
pub mod user;
mod user_list;
mod transfer;
mod name;

use axum::{Router, routing::{post, put}};
use super::{AppState, Jwt, Response};
use response::RoomResp;


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(my::route("/my"))
        .merge(user::route("/user"))
        .merge(create::route("/create"))
        .merge(detail::route("/detail"))
        .merge(transfer::route("/transfer"))
        .route("/{id}/name", put(name::update_room_name))
        .merge(user_list::route("/user_list"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}