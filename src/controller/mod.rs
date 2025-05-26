mod error;
mod response;
mod jwt;
mod http;
mod ws;

use std::sync::Arc;
use axum::Router;
use chrono::Duration;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

use jwt::Jwt;
use error::Error;
pub use response::Response;
use crate::controller::jwt::{JwtAuthMethod, JwtDomain};
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
    pub jwt_auth_method: JwtAuthMethod,
    pub jwt_secret: String,
    pub jwt_exp_duration: Duration,
    pub jwt_domain: JwtDomain,
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A,
    repo: Arc<dyn Repository>,
    jwt_secret: String,
    jwt_exp_duration: Duration,
) -> JoinHandle<Result<(), String>> {
    
    let http_state = AppState {
        repository: repo.clone(),
        jwt_auth_method: JwtAuthMethod::Cookie,
        jwt_secret: jwt_secret.clone(),
        jwt_exp_duration,
        jwt_domain: JwtDomain::Http,
    };
    
    let chat_ws_state = AppState {
        repository: repo.clone(),
        jwt_auth_method: JwtAuthMethod::Headers,
        jwt_secret,
        jwt_exp_duration,
        jwt_domain: JwtDomain::WebSocketChat,
    };
    
    let app = Router::new()
        .merge(http::route("/", http_state))
        .merge(ws::route("/ws", chat_ws_state));

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    })
}
 
