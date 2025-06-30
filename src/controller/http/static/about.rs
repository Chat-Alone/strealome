use crate::controller::Error;
use axum::{routing, Router};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use tokio::fs::read_to_string;
use crate::controller::AppState;
use crate::unwrap;

async fn get() -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/about.html").await);
    Html(str).into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}