use axum::{Router};
use super::{AppState, Jwt, Error, Response};

mod register;
mod index;
mod login;
mod logout;
mod profile;
mod gateway;

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(index::route("/"))
        .merge(login::route("/login"))
        .merge(logout::route("/logout"))
        .merge(register::route("/register"))
        .merge(profile::route("/user/profile"))
        .merge(gateway::route("/chat/gateway"))
        .with_state(app_state);
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
