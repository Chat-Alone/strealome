use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use super::AppState;

static ASSETS_PATH: &str = "frontend/public";

pub fn route(path: &str) -> Router<AppState> {
    let service = ServeDir::new(ASSETS_PATH)
        .not_found_service(ServeFile::new("frontend/404.html"));
    
    Router::new().nest_service(path, service)
}