mod login;
mod logout;
mod profile;
mod register;

use axum::Router;
use super::{Jwt, AppState, Error, Response};

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(login::route("/login"))
        .merge(logout::route("/logout"))
        .merge(profile::route("/profile"))
        .merge(register::route("/register"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}