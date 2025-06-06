mod error;
mod response;
mod jwt;
mod http;
mod ws;
mod webrtc;

use std::sync::Arc;
use axum::Router;
use chrono::Duration;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

use tower_http::{cors::CorsLayer, trace::TraceLayer};

use jwt::Jwt;
use error::Error;
pub use response::Response;
use crate::controller::jwt::{JwtAuthMethod, JwtDomain};
use crate::repository::Repository;
use crate::service::room::Rooms;

#[macro_export]
macro_rules! unwrap {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.into(),
        }
    };
}

#[macro_export]
macro_rules! read_to_string {
    ($result:expr) => {
        match fs::read_to_string($result).await {
            Ok(v) => v,
            Err(e) => return Error::from(e).into_response(),
        }
    };
}

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<dyn Repository>,
    pub rooms: Rooms,
    pub jwt_auth_method: JwtAuthMethod,
    pub jwt_secret: String,
    pub jwt_exp_duration: Duration,
    pub jwt_exp_dur_long: Duration,
    pub jwt_domain: JwtDomain,
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A,
    repo: Arc<dyn Repository>,
    jwt_secret: String,
    jwt_exp_duration: Duration,
    jwt_exp_dur_long: Duration
) -> JoinHandle<Result<(), String>> {
    
    let rooms = Rooms::new();
    
    let http_state = AppState {
        rooms: rooms.clone(),
        repository: repo.clone(),
        jwt_auth_method: JwtAuthMethod::Cookie,
        jwt_secret: jwt_secret.clone(),
        jwt_exp_duration,
        jwt_exp_dur_long,
        jwt_domain: JwtDomain::Http,
    };
    
    let chat_ws_state = AppState {
        rooms,
        repository: repo.clone(),
        jwt_auth_method: JwtAuthMethod::Query,
        jwt_secret,
        jwt_exp_duration,
        jwt_exp_dur_long,
        jwt_domain: JwtDomain::WebSocketChat,
    };
    
    let app = Router::new()
        .merge(http::route("/", http_state.clone()))
        .merge(webrtc::route("/s", http_state.clone()))
        .merge(ws::route("/ws", chat_ws_state));

    #[cfg(debug_assertions)]
    let app = app.layer(CorsLayer::permissive());

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    })
}
 
