use axum::{Router, routing};
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response as AxumResponse};

use super::{AppState, Response};
pub async fn post(State(state): State<AppState>) -> AxumResponse {
    if state.jwt_auth_method.is_cookie() {
        let mut res = Response::success::<()>(None).into_response();
        res.headers_mut()
            .insert(SET_COOKIE, HeaderValue::from_str(
                "token=nothing; Max-Age={}; Path=/; HttpOnly; Secure; SameSite=Strict"
            ).unwrap());
        return res
    }
    
    Response::success::<()>(None).into_response()
}


pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
