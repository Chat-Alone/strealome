use serde::{Deserialize, Serialize};

use axum::{Json, routing};
use axum::extract::{Query, State};
use axum::http::{HeaderValue, header::SET_COOKIE};
use axum::response::{IntoResponse, Redirect, Response as AxumResponse};

use super::{Jwt, AppState, Response};
use crate::service::user;

#[derive(Serialize, Deserialize, Debug)]
struct PostQuery {
    redirect: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct PostRequest {
    username: String,
    password: String,
    remember: bool,
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

async fn post(
    State(state): State<AppState>,
    Query(PostQuery {redirect}): Query<PostQuery>,
    Json(req): Json<PostRequest>
) -> AxumResponse {
    println!("{:?}", redirect);
    let remember = req.remember;
    match user::handle_login(state.repository, req.into()).await {
        Ok(user) => {
            let duration = if remember { state.jwt_exp_dur_long } else { state.jwt_exp_duration };
            let jwt = Jwt::http(user.id, duration);
            let token = jwt.encode(&state.jwt_secret).unwrap_or("wtf?".to_string());
            let res = if state.jwt_auth_method.is_cookie() {
                let cookie = format!(
                    "token={}; Max-Age={}; Path=/; HttpOnly; SameSite=Strict",
                    &token, duration.num_seconds()
                );
                let mut res = if let Some(redirect) = redirect {
                    Redirect::to(&redirect).into_response()
                } else {
                    Response::success::<()>(None).into_response()
                };
                res.headers_mut()
                    .insert(SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
                res
            } else {
                let post_res = PostResponse { token };
                if let Some(redirect) = redirect {
                    Redirect::to(&redirect).into_response()
                } else {
                    Response::success(Some(serde_json::to_value(&post_res).unwrap())).into_response()
                }
            };
            
            println!("{:?}", res);

            res
        },
        Err(e) => Response::from(e).into_response(),
    }
} 


pub fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new()
        .route(path, routing::post(post))
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