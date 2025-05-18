use tokio::fs::read_to_string;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json, Router,
};

use crate::unwrap;

pub struct ResponseError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

impl From<std::io::Error> for ResponseError {
    fn from(e: std::io::Error) -> Self {
        ResponseError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }
}


async fn get() -> Response {
    let str = unwrap!(read_to_string("frontend/register.html").await);
    Html(str).into_response()
}