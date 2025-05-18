use crate::model::UserModel;
use crate::repository::crud::CRUD;

#[async_trait::async_trait]
pub trait UserRepo: CRUD<Target = UserModel> {}