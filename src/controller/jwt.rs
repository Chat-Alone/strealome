use axum::extract::{FromRequest, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use axum::response::{IntoResponse, Response};
use axum_extra::{
    headers::{authorization::Bearer, Authorization, Cookie},
    TypedHeader,
};

use chrono::Duration;
use jsonwebtoken::errors::Result;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::USE_COOKIE;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
}

impl Jwt {
    pub fn new(sub: i32, exp_duration_s: Duration) -> Self {
        let iat = chrono::Local::now().timestamp();
        let exp = iat + exp_duration_s.num_seconds();
        Self {
            sub,
            exp,
            iat
        }
    }
    
    pub fn verify(&self) -> bool {
        chrono::Local::now().timestamp() < self.exp
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
    type Rejection = Response;

    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=std::result::Result<Self, Self::Rejection>> + Send {
        async move {
            if USE_COOKIE {
                if let Ok(TypedHeader(cookie)) = req.extract::<TypedHeader<Cookie>>().await {
                    if let Some(token) = cookie.get("token") {
                        if let Ok(jwt) = Jwt::decode(token, &state.jwt_secret) {
                            return jwt.verify()
                                .then_some(jwt)
                                .ok_or((StatusCode::UNAUTHORIZED,
                                        "Invalid token".to_string()).into_response())
                        }
                    }
                }
                Err((StatusCode::UNAUTHORIZED).into_response())
            }

            else {
                if let Ok(TypedHeader(bearer)) = req.extract::<TypedHeader<Authorization<Bearer>>>().await {
                    if let Ok(jwt) = Jwt::decode(bearer.token(), &state.jwt_secret) {
                        return jwt.verify()
                            .then_some(jwt)
                            .ok_or((StatusCode::UNAUTHORIZED,
                                    "Invalid token".to_string()).into_response())
                    }
                }
                Err((StatusCode::UNAUTHORIZED).into_response())
            }
        }
    }
}

impl OptionalFromRequestParts<AppState> for Jwt {
    type Rejection = (StatusCode, String);

    // NEVER REJECT
    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=std::result::Result<Option<Self>, Self::Rejection>> + Send {
        async move {
            if USE_COOKIE {
                if let Ok(TypedHeader(cookie)) = req.extract::<TypedHeader<Cookie>>().await {
                    println!("Cookie: {:?}", cookie);
                    if let Some(token) = cookie.get("token") {
                        println!("Token: {:?}", token);
                        if let Ok(jwt) = Jwt::decode(token, &state.jwt_secret) {
                            println!("JWT: {:?}", jwt);
                            return Ok(jwt.verify().then_some(jwt))
                        }
                    }
                }
            }

            else {
                if let Ok(TypedHeader(bearer)) = req.extract::<TypedHeader<Authorization<Bearer>>>().await {
                    if let Ok(jwt) = Jwt::decode(bearer.token(), &state.jwt_secret) {
                        return Ok(jwt.verify().then_some(jwt))
                    }
                }
            }
            
            Ok(None)
        }
    }
}

