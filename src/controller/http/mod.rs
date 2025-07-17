mod room;
mod chat;
mod user;
mod r#static;

use axum::Router;
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use axum::http::StatusCode;
use axum::extract::State;
use tokio::fs::read_to_string;
use super::{AppState, Error, Jwt, Response};
use crate::unwrap;

async fn fallback_404(State(_state): State<AppState>) -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/404.html").await);
    (StatusCode::NOT_FOUND, Html(str)).into_response()
}

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(r#static::route("/"))
        .merge(chat::route("/chat"))
        .merge(user::route("/user"))
        .merge(room::route("/room"))
        .fallback(fallback_404)
        .with_state(app_state);
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
