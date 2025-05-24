use axum::{Router, routing};
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response as AxumResponse};

use crate::{USE_COOKIE};
use super::{AppState, Response};
async fn post() -> AxumResponse {
    if USE_COOKIE {
        let mut res = Response::success(None).into_response();
        res.headers_mut()
            .insert(SET_COOKIE, HeaderValue::from_str("token=").unwrap());
        return res
    }
    
    Response::success(None).into_response()
}


pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
