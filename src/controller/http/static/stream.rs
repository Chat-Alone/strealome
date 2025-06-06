use axum::{Router, routing};
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response as AxumResponse};
use tokio::fs;
use crate::service::user;
use crate::{read_to_string};
use super::{Error, AppState, Jwt};

async fn get(jwt: Option<Jwt>, State(state): State<AppState>) -> AxumResponse {
    if jwt.is_none() {
        return Redirect::to("/").into_response()
    }
    let jwt = jwt.unwrap();
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if user.is_err() {
        return Redirect::to("/").into_response()
    }
    let user = user.unwrap();
    let str = read_to_string!("frontend/stream.html");
    let str = str.replace("{{USERNAME}}", &user.name);
    Html(str).into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    let path = if path == "/" { "/{room}" } else { &format!("{path}/{{room}}") };
    Router::new()
        .route(path, routing::get(get))
}