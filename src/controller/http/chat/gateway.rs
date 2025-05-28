use axum::extract::State;
use axum::Router;
use serde::Serialize;

use crate::service::user;
use crate::controller::{Jwt, AppState, Response};

#[derive(Serialize)]
struct GetResponse {
    gateway_token: String,
}

async fn get(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();

    let gateway_token = Jwt::chat_ws(user.id, state.jwt_exp_duration);
    let gateway_token = match gateway_token.encode(&state.jwt_secret) {
        Ok(token) => token,
        Err(_) => return Response::error("Failed to encode gateway token"),
    };
    Response::success(Some(GetResponse { gateway_token }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::get(get))
}