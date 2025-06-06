pub mod user;
pub mod chat;
pub mod room;
pub mod webrtc;
mod error;

pub use error::Error;

use crate::model::user::UserModel;
use crate::controller::Response;
use crate::repository::{CRUD, Repository, UserRepo};
