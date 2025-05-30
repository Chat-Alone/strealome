use std::collections::HashMap;
use axum::extract::{FromRequest, FromRequestParts, OptionalFromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;
use axum::response::{IntoResponse, Response};
use axum_extra::{
    headers::{authorization::Bearer, Authorization, Cookie},
    TypedHeader,
};

use chrono::Duration;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize, Serializer, Deserializer};

use super::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtDomain {
    Http,
    WebSocketChat,
    WebSocketStream,
}


impl Serialize for JwtDomain {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}
impl<'de> Deserialize<'de> for JwtDomain {
    fn deserialize<D:Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let u8 = u8::deserialize(deserializer)?;
        match u8 {
            0 => Ok(JwtDomain::Http),
            1 => Ok(JwtDomain::WebSocketChat),
            2 => Ok(JwtDomain::WebSocketStream),
            _ => Err(serde::de::Error::custom("Invalid domain"))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
    pub dom: JwtDomain,
}

impl Jwt {
    pub fn new(sub: i32, exp_duration_s: Duration, domain: JwtDomain) -> Self {
        let iat = chrono::Local::now().timestamp();
        let exp = iat + exp_duration_s.num_seconds();
        Self {
            sub,
            exp,
            iat,
            dom: domain,
        }
    }
    
    pub fn http(sub: i32, exp_duration_s: Duration) -> Self {
        Self::new(sub, exp_duration_s, JwtDomain::Http)
    }
    
    pub fn chat_ws(sub: i32, exp_duration_s: Duration) -> Self {
        Self::new(sub, exp_duration_s, JwtDomain::WebSocketChat)
    }
    
    pub fn stream_ws(sub: i32, exp_duration_s: Duration) -> Self {
        Self::new(sub, exp_duration_s, JwtDomain::WebSocketStream)
    }
    
    pub fn verify(&self, domain: JwtDomain) -> bool {
        domain == self.dom && chrono::Local::now().timestamp() < self.exp
    }
    
    pub fn encode(&self, secret: &str) -> Result<String, Error> {
        let header = Header::new(jsonwebtoken::Algorithm::HS256);
        let key = EncodingKey::from_secret(secret.as_bytes());
        let ret = encode(&header, self, &key)?;
        Ok(ret)
    }
    
    pub fn decode(token: &str, secret: &str) -> Result<Self, Error> {
        let key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        jsonwebtoken::decode::<Self>(token, &key, &validation).map(|data| data.claims)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtAuthMethod {
    Cookie, Headers, Query,
}

impl JwtAuthMethod {
    pub fn is_cookie(&self) -> bool {
        self == &JwtAuthMethod::Cookie
    }
    
    pub fn is_headers(&self) -> bool {
        self == &JwtAuthMethod::Headers
    }
}

impl FromRequestParts<AppState> for Jwt {
    type Rejection = Response;

    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=Result<Self, Self::Rejection>> + Send {
        async move {
            let token_str = match state.jwt_auth_method {
                JwtAuthMethod::Cookie => {
                    match req.extract::<TypedHeader<Cookie>>().await {
                        Ok(TypedHeader(cookie)) 
                            => cookie.get("token").map(|s| s.to_string()),
                        Err(_) => None,
                    }
                },
                
                JwtAuthMethod::Headers => {
                    match req.extract::<TypedHeader<Authorization<Bearer>>>().await {
                        Ok(TypedHeader(bearer)) 
                            => Some(bearer.token().to_string()),
                        Err(_) => None,
                    }
                },

                JwtAuthMethod::Query => {
                    match req.extract::<Query<HashMap<String, String>>>().await {
                        Ok(Query(query)) 
                            => query.get("token").map(|s| s.to_string()),
                        Err(_) => None,
                    }
                }
            };
            
            
            if let Some(token_str) = token_str {
                if let Ok(jwt) = Jwt::decode(&token_str, &state.jwt_secret) {
                    return jwt.verify(state.jwt_domain)
                        .then_some(jwt)
                        .ok_or((StatusCode::UNAUTHORIZED,
                                "Invalid token".to_string()).into_response());
                }
            }

            Err((StatusCode::UNAUTHORIZED).into_response())
        }
    }
}

impl OptionalFromRequestParts<AppState> for Jwt {
    type Rejection = (StatusCode, String);

    // NEVER REJECT
    fn from_request_parts(req: &mut Parts, state: &AppState) -> impl Future<Output=Result<Option<Self>, Self::Rejection>> + Send {
        async move {
            match state.jwt_auth_method {
                JwtAuthMethod::Cookie => {
                    if let Ok(TypedHeader(cookie)) = req.extract::<TypedHeader<Cookie>>().await {
                        if let Some(token) = cookie.get("token") {
                            if let Ok(jwt) = Jwt::decode(token, &state.jwt_secret) {
                                return Ok(jwt.verify(state.jwt_domain).then_some(jwt))
                            }
                        }
                    }
                },
                
                JwtAuthMethod::Headers => {
                    if let Ok(TypedHeader(bearer)) = req.extract::<TypedHeader<Authorization<Bearer>>>().await {
                        if let Ok(jwt) = Jwt::decode(bearer.token(), &state.jwt_secret) {
                            return Ok(jwt.verify(state.jwt_domain).then_some(jwt))
                        }
                    }
                }
                
                JwtAuthMethod::Query => {
                    if let Ok(Query(query)) = req.extract::<Query<HashMap<String, String>>>().await {
                        if let Some(token) = query.get("token") {
                            if let Ok(jwt) = Jwt::decode(token, &state.jwt_secret) {
                                return Ok(jwt.verify(state.jwt_domain).then_some(jwt))
                            }
                        }
                    }
               }
            }
            
            Ok(None)
        }
    }
}

