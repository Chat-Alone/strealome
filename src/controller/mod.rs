mod error;
mod response;
mod jwt;
mod http;

use std::sync::Arc;
use axum::Router;
use chrono::Duration;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

use jwt::Jwt;
use error::Error;
pub use response::Response;
use crate::repository::Repository;

#[macro_export]
macro_rules! unwrap {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return Error::from(e).into_response(),
        }
    };
}

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<dyn Repository>,
    pub jwt_secret: String,
    pub jwt_exp_duration: Duration
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A, repository: Arc<dyn Repository>, jwt_secret: String, jwt_exp_duration: Duration
) -> JoinHandle<Result<(), String>> {
    let app = Router::new()
        .merge(http::route("/"))
        .with_state(AppState { repository, jwt_secret, jwt_exp_duration });

    let listener = TcpListener::bind(addr).await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    })
}
 
