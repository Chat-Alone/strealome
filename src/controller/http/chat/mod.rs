mod message;
mod gateway;

use axum::Router;
use super::{AppState, Jwt, Response};

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(message::route("/message"))
        .merge(gateway::route("/gateway"));
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}