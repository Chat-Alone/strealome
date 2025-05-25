use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use axum::{Json, Router, routing};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use serde_json::json;

use crate::unwrap;
use super::{AppState, Error, Response};
use crate::service::user;

#[derive(Serialize, Deserialize, Debug)]
struct PostRequest {
    username: String,
    password: String,
}

impl From<PostRequest> for user::RegisterParam {
    fn from(params: PostRequest) -> Self {
        Self {
            username: params.username,
            password: params.password,
        }
    }
}

async fn get() -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/register.html").await);
    Html(str).into_response()
}

async fn post(Json(param): Json<PostRequest>) -> Response {
    let param = param.into();
    let user = user::handle_register(param).await;
    if let Err(e) = user {
        return e.into();
    }
    let user = user.unwrap();
    Response::success(Some(json!{{ "id": user.id }}))
}

pub fn route(path: &str) -> Router<AppState> { 
    Router::new().route(path, routing::get(get).post(post))
}
