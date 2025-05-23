use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use axum::Json;
use axum::response::{Html, IntoResponse, Response as AxumResponse};
use crate::controller::error::Error;
use crate::controller::response::Response;
use crate::unwrap;
use crate::service::user;
use crate::service::user::UserError;

#[derive(Serialize, Deserialize, Debug)]
struct PostParams {
    username: String,
    password: String,
}

impl From<PostParams> for user::RegisterParam {
    fn from(params: PostParams) -> Self {
        Self {
            username: params.username,
            password: params.password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PostResponse {
    
}

async fn get() -> AxumResponse {
    let str = unwrap!(read_to_string("frontend/register.html").await);
    Html(str).into_response()
}

async fn post(Json(param): Json<PostParams>) -> Response {
    let param = param.into();
    
    let user = user::handle_register(&param).await;
    user.into()
}
