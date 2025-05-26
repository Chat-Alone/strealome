use serde::{Deserialize, Serialize};
use axum::{Json, Router, routing};
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use serde_json::json;

use crate::unwrap;
use super::{Jwt, AppState, Response};
use crate::service::user;

#[derive(Serialize, Deserialize, Debug)]
struct PostRequest {
    #[serde(rename = "currentPassword")]
    current_password: Option<String>,
    #[serde(rename = "newPassword")]
    new_password: Option<String>,
    #[serde(rename = "newUsername")]
    new_username: Option<String>,
}

impl From<PostRequest> for user::UpdateProfileParam {
    fn from(req: PostRequest) -> Self {
        Self {
            old_password: req.current_password,
            new_password: req.new_password,
            new_username: req.new_username,
        }
    }
}

async fn put(jwt: Jwt, Json(req): Json<PostRequest>) -> AxumResponse {
    let res = user::update_profile(jwt.sub, req.into()).await;

    match res {
        Ok(updated) => Response::success(Some(updated)).into_response(),
        Err(e) => Response::from(e).into_response(),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::put(put))
}
