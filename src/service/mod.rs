pub mod user;
pub mod chat;
pub mod room;
mod error;

pub use error::Error;

use crate::model::UserModel;
use crate::controller::Response;
use crate::repository::{CRUD, Repository, UserRepo};
