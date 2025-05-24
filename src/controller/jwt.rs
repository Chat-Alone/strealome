use axum::extract::{FromRequest, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use jsonwebtoken::errors::Result;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
}

impl Jwt {
    pub fn new(sub: i32, exp: i64, iat: i64) -> Self {
        Self { sub, exp, iat }
    }
    
    pub fn verify(&self) -> bool {
        chrono::Local::now().timestamp() > self.exp
    }
    
    pub fn encode(&self, secret: &str) -> Result<String> {
        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        let key = EncodingKey::from_secret(secret.as_bytes());
        let ret = encode(&header, self, &key)?;
        Ok(ret)
    }
    
    pub fn decode(token: &str, secret: &str) -> Result<Self> {
        let key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        jsonwebtoken::decode::<Self>(token, &key, &validation).map(|data| data.claims)
    }
}

impl FromRequestParts<AppState> for Jwt {
    type Rejection = (StatusCode, String);

    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=std::result::Result<Self, Self::Rejection>> + Send {
        async move {
            let header = req.headers.get("Authorization");
            if header.is_none() {
                return Err((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()));
            }
            let header = header.unwrap();
            
            if let Ok(header) = header.to_str() {
                if let Some(bearer) = header.strip_prefix("Bearer ") {
                    if let Ok(jwt) = Jwt::decode(bearer, &state.jwt_secret) {
                        if jwt.verify() {
                            return Ok(jwt)
                        }
                    }
                }
            }
            Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header".to_string()))
        }
    }
}

impl OptionalFromRequestParts<AppState> for Jwt {
    type Rejection = (StatusCode, String);

    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=std::result::Result<Option<Self>, Self::Rejection>> + Send {
        async move {
            if let Some(header) = req.headers.get("Authorization") {
                if let Ok(header) = header.to_str() {
                    if let Some(bearer) = header.strip_prefix("Bearer ") {
                        if let Ok(jwt) = Jwt::decode(bearer, &state.jwt_secret) {
                            return Ok(jwt.verify().then_some(jwt))
                        }
                    }
                }
            }
            
            Ok(None)
        }
    }
}

