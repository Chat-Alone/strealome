use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use axum::{Json, Router, routing};
use axum::extract::State;
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

async fn post(State(state): State<AppState>, Json(param): Json<PostRequest>) -> Response {
    let param = param.into();
    let user = user::handle_register(state.repository, param).await;
    if let Err(e) = user {
        return e.into();
    }
    let user = user.unwrap();
    Response::success(Some(json!{{ "id": user.id }}))
}

pub fn route(path: &str) -> Router<AppState> { 
    Router::new().route(path, routing::post(post))
}
