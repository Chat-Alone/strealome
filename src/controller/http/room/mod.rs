mod my;
mod create;
mod response;
mod detail;
pub mod user;
mod user_list;

use axum::Router;
use super::{AppState, Jwt, Response};
use response::RoomResp;


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(my::route("/my"))
        .merge(user::route("/user"))
        .merge(user_list::route("/user_list"))
        .merge(detail::route("/detail"))
        .merge(create::route("/create"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}