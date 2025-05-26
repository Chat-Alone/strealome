use axum::http::StatusCode;
use thiserror::Error as ThisError;
use crate::controller::Response;
use crate::repository;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}


impl From<Error> for Response {
    fn from(e: Error) -> Self {
        match e {
            Error::RepositoryError(e) => Response::fail::<()>(StatusCode::INTERNAL_SERVER_ERROR, None),
        }
    }
}