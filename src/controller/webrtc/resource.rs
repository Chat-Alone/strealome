use axum::http::StatusCode;
use axum::{routing, Json, Router};
use axum::extract::{Request};
use axum::http::header;
use axum::response::{IntoResponse, Response as AxumResponse};
use dashmap::DashMap;
use serde::Deserialize;
use super::{Response, Jwt, AppState};


async fn patch(
    // jwt: Jwt,  State(s): State<AppState>,
    req: Request,
) -> AxumResponse {
    // let user_id = jwt.sub;
    // unwrap!(user::get_user_by_id(s.repository, user_id).await);
    //
    if !req.headers().get("content-type")
        .map(|h| {
            h.as_bytes() == "application/trickle-ice-sdpfrag".as_bytes()
        }).unwrap_or(false)
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    
    StatusCode::METHOD_NOT_ALLOWED.into_response()
    
    // let if_match = req.headers().get(header::IF_MATCH);
    // if let None = if_match {
    //     return StatusCode::PRECONDITION_FAILED.into_response()
    // }
    // let if_match = if_match.unwrap().to_str().unwrap();
    // match if_match {
    //     "*" => {}
    //     etag => {
    //         if RESOURCES.get(req.uri().path()).unwrap().as_str() != etag {
    //             return StatusCode::PRECONDITION_FAILED.into_response()
    //         }
    //     }
    // }
    // 
    // StatusCode::OK.into_response()
}

#[derive(Deserialize)]
struct PostRequest {
    name: String,
}

async fn post(
    // jwt: Jwt,  State(s): State<AppState>,
    Json(req): Json<PostRequest>
) -> AxumResponse { 
    StatusCode::OK.into_response()
}

async fn delete(
    // jwt: Jwt,  State(s): State<AppState>,
) -> AxumResponse {
    StatusCode::OK.into_response()
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::patch(patch).post(post).delete(delete))
}