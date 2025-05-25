use thiserror::Error as ThisError;

use crate::REPO;
use crate::controller::Response;
use crate::model::UserModel;
use crate::repository::{CRUD, Repository, UserRepo};

#[derive(ThisError, Debug)]
pub enum UserError {
    #[error("Username must be 4 to 16 characters long.")]
    InvalidUsername,

    #[error("Password must be 8 to 32 characters long and contain at least one digit and one letter.")]
    InvalidPassword,

    #[error("User already exists.")]
    UserAlreadyExists,

    #[error("Incorrect username or password.")]
    IncorrectCredentials,
    
    #[error("Service error")]
    ServiceError(#[from] super::Error)
}

impl From<UserError> for Response {
    fn from(e: UserError) -> Self {
        eprintln!("UserError: {e}");
        Response::error(&e.to_string())
    }
}

fn validate_username(username: &str) -> bool {
    username.len() >= 4 && username.len() <= 16
}

fn validate_password(password: &str) -> bool {
    if password.len() < 8 || password.len() > 32 { return false }
    let mut has_digit = false;
    let mut has_letter = false;
    for c in password.chars() {
        if !c.is_ascii_alphanumeric() { return false }
        if !has_digit && c.is_ascii_digit() { has_digit = true }
        else if !has_letter && c.is_ascii_alphabetic() { has_letter = true }
    }
    
    has_digit && has_letter
}

fn bcrypt_password(password: &str, cost: u32) -> String {
    bcrypt::hash(password, cost).unwrap()
}

fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap()
}

pub struct RegisterParam {
    pub username: String,
    pub password: String,
}

pub async fn handle_register(param: RegisterParam) -> Result<UserModel, UserError> {
    let RegisterParam { username, password } = param;
    if !validate_username(&username) {
        return Err(UserError::InvalidUsername);
    };
    if !validate_password(&password) {
        return Err(UserError::InvalidPassword);
    };
    
    let conn = REPO.clone().await;
    let user = conn.find_by_name(&username).await.map_err(|e| super::Error::from(e))?;
    if user.is_some() {
        return Err(UserError::UserAlreadyExists);
    };
    
    let crypted_password = bcrypt_password(&password, bcrypt::DEFAULT_COST);
    let user = conn.create(UserModel::new_user(username, crypted_password)).await;
    Ok(user)
}

pub struct LoginParam {
    pub username: String,
    pub password: String,
}

pub async fn handle_login(param: LoginParam) -> Result<UserModel, UserError> {
    let LoginParam { username, password } = param;
    if !validate_username(&username) {
        return Err(UserError::InvalidUsername);
    };
    if !validate_password(&password) {
        return Err(UserError::InvalidPassword);
    };
    
    let conn = REPO.clone().await;
    let user = conn.find_by_name(&username).await.map_err(|e| super::Error::from(e))?;
    if let Some(user) = user {
        if verify_password(&password, &user.password) {
            return Ok(user)
        }
    }
    
    Err(UserError::IncorrectCredentials)
}