use axum::{Router, routing};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use tokio::fs;
use crate::{read_to_string};
use super::{Error, AppState};

async fn get() -> AxumResponse {
    let str = read_to_string!("frontend/register.html");
    Html(str).into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::get(get))
}