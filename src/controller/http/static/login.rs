use crate::controller::Error;
use axum::{Router, routing};
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response as AxumResponse};

use tokio::fs::read_to_string;

use crate::unwrap;
use super::{AppState, Jwt};

async fn get(jwt: Option<Jwt>, State(state): State<AppState>) -> AxumResponse {
    if let Some(jwt) = jwt {
        let user = state.repository.find_by_id(jwt.sub).await;
        if user.is_some() {
            let str = unwrap!(read_to_string("frontend/index.html").await);
            return Html(str).into_response();
        }
    }
    let str = unwrap!(read_to_string("frontend/login.html").await);
    Html(str).into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::get(get))
}