use serde::{Deserialize, Serialize};
use axum::{Json, Router, routing};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use serde_json::json;

use crate::unwrap;
use super::{Jwt, AppState, Error, Response};
use crate::service::user;
use super::logout;

#[derive(Serialize, Deserialize, Debug)]
struct PostRequest {
    password: String,
}

async fn post(jwt: Jwt, Json(param): Json<PostRequest>) -> AxumResponse {
    let PostRequest { password } = param;
    let res = user::change_password(jwt.sub, &password).await;
    if let Err(e) = res {
        return Response::from(e).into_response();
    }
    
    logout::post().await
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
