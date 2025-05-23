use axum::extract::State;
use crate::controller::error::Error;
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use tokio::fs::read_to_string;

use super::{Jwt};
use super::AppState;

use crate::unwrap;

async fn get(jwt: Option<Jwt>, State(state): State<AppState>) -> AxumResponse {
    println!("{jwt:?}");
    if let Some(jwt) = jwt {
        let user = state.repository.find_by_id(jwt.sub).await;
        if let Some(user) = user {
            let str = unwrap!(read_to_string("frontend/index.html").await);
            return Html(str).into_response();
        }
    }
    let str = unwrap!(read_to_string("frontend/login.html").await);
    Html(str).into_response()
}

pub fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::get(get))
}