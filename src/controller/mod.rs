mod error;
mod register;
mod response;
mod root;
mod jwt;
mod login;

use std::sync::Arc;
use axum::Router;

use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

use jwt::Jwt;
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
    pub jwt_secret: String
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A, repository: Arc<dyn Repository>, jwt_secret: String
) -> JoinHandle<Result<(), String>> {
    let app = Router::new()
        .merge(root::route("/"))
        .merge(login::route("/login"))
        .merge(register::route("/register"))
        .with_state(AppState { repository, jwt_secret });

    let listener = TcpListener::bind(addr).await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    })
}
 
