use axum::{Router, routing};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use tokio::fs::read_to_string;
use crate::unwrap;
use super::{Error, AppState};

async fn get() -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/stream.html").await);
    Html(str).into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    let path = if path == "/" { "/{room}" } else { &format!("{path}/{{room}}") };
    Router::new()
        .route(path, routing::get(get))
}