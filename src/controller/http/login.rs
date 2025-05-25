use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderValue, header::SET_COOKIE};
use axum::response::{Html, IntoResponse, Response as AxumResponse};

use super::{Jwt, AppState, Error, Response};
use crate::{unwrap, USE_COOKIE};
use crate::service::user;

#[derive(Serialize, Deserialize, Debug)]
struct PostRequest {
    username: String,
    password: String,
}

impl From<PostRequest> for user::LoginParam {
    fn from(params: PostRequest) -> Self {
        Self {
            username: params.username,
            password: params.password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PostResponse {
    pub token: String,
}


async fn get(jwt: Option<Jwt>, State(state): State<AppState>) -> AxumResponse {
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

async fn post(State(state): State<AppState>, Json(req): Json<PostRequest>) -> AxumResponse {
    
    match user::handle_login(req.into()).await {
        Ok(user) => {
            let jwt = Jwt::new(user.id, state.jwt_exp_duration);
            let token = jwt.encode(&state.jwt_secret).unwrap_or("wtf?".to_string());
            let post_res = PostResponse { token };
            let mut res = Response::success(Some(serde_json::to_value(&post_res).unwrap())).into_response();
            
            if USE_COOKIE {
                let cookie = format!(
                    "token={}; Max-Age={}; Path=/; HttpOnly; Secure; SameSite=Strict",
                    &post_res.token, state.jwt_exp_duration.num_seconds()
                );
                res.headers_mut()
                    .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
            }
            res
        },
        Err(e) => Response::from(e).into_response(),
    }
} 


pub fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::get(get).post(post))
}

// async fn post(jwt: Option<Jwt>, State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
//     let jwt_res = |user_id: i32| {
//         let jwt = Jwt::new(user_id, state.jwt_exp_duration);
//         let token = jwt.encode(&state.jwt_secret).unwrap_or("wtf?".to_string());
//         let res = PostResponse { token };
//         Response::success(Some(serde_json::to_value(res).unwrap()))
//     };
// 
//     if let Some(jwt) = jwt { // update jwt
//         if let Some(user) = state.repository.find_by_id(jwt.sub).await {
//             return jwt_res(user.id);
//         }
//     }
// 
//     match user::handle_login(req.into()).await {
//         Ok(user) => jwt_res(user.id),
//         Err(e) => e.into(),
//     }
// }