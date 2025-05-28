use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use serde_json::Value;
use thiserror::Error as ThisError;

use super::Response;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0:?}")]
    Genetic(Option<Value>),
    
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),
    
    #[error("Jwt Error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::IO(_)                => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JSON(_)              => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Jwt(_)               => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidArgument(_)   => StatusCode::BAD_REQUEST,
            Error::Genetic(_)           => StatusCode::OK,
        }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::Genetic(payload) => Response::fail(StatusCode::OK, payload),
            _ => Response::fail::<()>(e.status_code(), None),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let response = Response::from(self);
        response.into_response()
    }
}
