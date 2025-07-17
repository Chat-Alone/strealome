use axum::Router;
use super::{AppState, Jwt, Error, r#static};

mod index;
mod login;
mod register;
mod stream;
mod public;
mod about;
mod not_found;
mod room_not_found;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(index::route("/"))
        .merge(stream::route("/share"))
        .merge(login::route("/login"))
        .merge(register::route("/register"))
        .merge(about::route("/about"))
        .merge(not_found::route("/404"))
        .merge(room_not_found::route("/room-not-found"))
        .merge(public::route("/public"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}