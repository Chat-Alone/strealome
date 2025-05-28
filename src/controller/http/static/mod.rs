use axum::Router;
use super::{ AppState, Jwt, Error };

mod index;
mod login;
mod register;
mod stream;

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(index::route("/"))
        .merge(stream::route("/share"))
        .merge(login::route("/login"))
        .merge(register::route("/register"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}