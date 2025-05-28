use serde::{Deserialize, Serialize};
use axum::{Json, Router, routing};
use axum::extract::State;
use crate::model::UserModel;
use super::{Jwt, AppState, Response};
use crate::service::user;

#[derive(Serialize, Deserialize, Debug)]
struct PutRequest {
    #[serde(rename = "currentPassword")]
    current_password: Option<String>,
    #[serde(rename = "newPassword")]
    new_password: Option<String>,
    #[serde(rename = "newUsername")]
    new_username: Option<String>,
}

impl From<PutRequest> for user::UpdateProfileParam {
    fn from(req: PutRequest) -> Self {
        Self {
            old_password: req.current_password,
            new_password: req.new_password,
            new_username: req.new_username,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GetResponse {
    username: String,
}

impl From<UserModel> for GetResponse {
    fn from(user: UserModel) -> Self {
        Self {
            username: user.name,
        }
    }
}

async fn get(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    match user {
        Ok(user) => Response::success(Some(serde_json::to_value(GetResponse::from(user)).expect("wtf"))),
        Err(e) => e.into(),
    }
}

async fn put(jwt: Jwt, State(state): State<AppState>, Json(req): Json<PutRequest>) -> Response {
    let res = user::update_profile(state.repository, jwt.sub, req.into()).await;
    res.into()

}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get).put(put))
}
