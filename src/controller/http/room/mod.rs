mod my;
mod create;
mod response;
mod detail;

use axum::Router;
use super::{AppState, Response, Jwt};
use response::RoomResp;


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(my::route("/my"))
        .merge(detail::route("/detail"))
        .merge(create::route("/create"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}