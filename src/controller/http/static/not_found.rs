use axum::extract::State;
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use axum::http::StatusCode;
use tokio::fs::read_to_string;

use crate::controller::http::{AppState, Error};
use crate::unwrap;

async fn get(State(_state): State<AppState>) -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/404.html").await);
    (StatusCode::NOT_FOUND, Html(str)).into_response()
}

pub fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::get(get))
}
