mod chat;

use axum::Router;
use super::{AppState, Jwt, Response, Error};

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(chat::route("/"))
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}