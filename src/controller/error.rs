use std::error::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorResponse {
    #[error("DuckDB error: {0}")]
    DuckDB(#[from] duckdb::Error),
    
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
}

impl ErrorResponse {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponse::DuckDB(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::IO(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::JSON(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponse::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        }
    }
    
    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.status_code(), self.message()).into_response()
    }
}