use crate::model::user::UserModel;
use crate::repository::crud::CRUD;
use crate::repository::Error;

#[async_trait::async_trait]
pub trait UserRepo: CRUD<Target = UserModel, Error = Error> {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserModel>, Error>;
}