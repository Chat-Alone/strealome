use tokio::fs;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Response as AxumResponse};

use crate::controller::http::{Jwt, AppState, Error};
use crate::{read_to_string};

async fn get(jwt: Option<Jwt>, State(state): State<AppState>) -> AxumResponse {
    if let Some(jwt) = jwt {
        let user = state.repository.find_by_id(jwt.sub).await;
        if let Some(user) = user {
            let str = read_to_string!("frontend/index.html");
            return Html(str).into_response();
        }
    }
    let str = read_to_string!("frontend/login.html");
    Html(str).into_response()
}

pub fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::get(get))
}