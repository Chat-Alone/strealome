use axum::{Router};
use super::{AppState, Jwt, Error, Response};

mod register;
mod index;
mod login;
mod logout;
mod password_change;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(index::route("/"))
        .merge(login::route("/login"))
        .merge(logout::route("/logout"))
        .merge(register::route("/register"))
        .merge(password_change::route("/password/change"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
