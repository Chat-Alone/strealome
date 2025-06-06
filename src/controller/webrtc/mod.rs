mod whip;
mod resource;

use axum::Router;
use super::{AppState, Error, Jwt, Response};

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(whip::route("/whip"))
        .merge(resource::route("/res"))
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
