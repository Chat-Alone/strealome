mod room;
mod chat;
mod user;
mod r#static;

use axum::Router;
use super::{AppState, Error, Jwt, Response};

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(r#static::route("/"))
        .merge(chat::route("/chat"))
        .merge(user::route("/user"))
        .merge(room::route("/room"))
        .with_state(app_state);
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
