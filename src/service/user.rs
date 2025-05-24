use thiserror::Error as ThisError;

use crate::REPO;
use crate::controller::Response;
use crate::model::UserModel;
use crate::repository::{CRUD, Repository, UserRepo};

pub struct RegisterParam {
    pub username: String,
    pub password: String,
}

#[derive(ThisError, Debug)]
pub enum UserError {
    #[error("Invalid username")]
    InvalidUsername,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Username already exists")]
    ServiceError(#[from] super::Error)
}

impl From<UserError> for Response {
    fn from(e: UserError) -> Self {
        match e {
            _ => Response::error(&e.to_string()),
        }
    }
}

fn verify_username(username: &str) -> bool {
    username.len() >= 4 && username.len() <= 16
}

fn verify_password(password: &str) -> bool {
    let regex = regex::Regex::new(r"^(?=.*[a-z])(?=.*\d).{8,32}$").unwrap();
    regex.is_match(password)
}

pub async fn handle_register(param: &RegisterParam) -> Result<UserModel, UserError> {
    let RegisterParam { username, password } = param;
    if !verify_username(username) {
        return Err(UserError::InvalidUsername);
    };
    if !verify_password(password) {
        return Err(UserError::InvalidPassword);
    };
    
    let conn = REPO.clone().await;
    let user = conn.find_by_name(username).await.map_err(|e| super::Error::from(e))?;
    if user.is_some() {
        return Err(UserError::UserAlreadyExists);
    };
    
    let user = conn.create(UserModel::new_user(username.to_string(), password.to_string())).await;
    Ok(user)
}