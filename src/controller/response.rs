use std::sync::atomic::AtomicU32;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{ IntoResponse, Response as AxumResponse };
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

static ID: AtomicU32 = AtomicU32::new(0);

fn get_id() -> u32 {
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip)]
    status_code: StatusCode,
    
    id: u32,
    success: bool,
    payload: Option<Value>,
    created_at: DateTime<Utc>,
}

impl Response {
    pub fn new(id: u32, success: bool, status_code: StatusCode, payload: Option<Value>) -> Self {
        Self {
            status_code,
            
            id,
            success,
            payload,
            created_at: Utc::now(),
        }
    }
    
    pub fn success(payload: Option<Value>) -> Self {
        Self::new(get_id(), true, StatusCode::OK, payload)
    }
    
    pub fn code(status_code: StatusCode) -> Self {
        Self::new(get_id(), status_code == StatusCode::OK, status_code, None)
    }
    
    pub fn fail(status_code: StatusCode, payload: Option<Value>) -> Self {
        Self::new(get_id(), false, status_code, payload)
    }
    
    pub fn error(description: &str) -> Self {
        Self::fail(StatusCode::OK, Some(json!({ "error": description })))
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> AxumResponse {
        if self.status_code.is_success() {
            return (self.status_code, Json(self)).into_response();
        } 
        (self.status_code, self.status_code.to_string()).into_response()
    }
}

impl<T, E> From<Result<T, E>> for Response where T: Serialize, E: Into<Response> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Response::success(Some(json!(v))),
            Err(e) => e.into(),
        }
    }
}