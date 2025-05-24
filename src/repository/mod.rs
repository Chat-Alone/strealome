mod user;
mod duckdb_impl;
mod crud;
mod config;
mod error;

pub use crud::CRUD;
pub use error::Error;
pub use config::RepoConfig;
pub use user::UserRepo;
pub use duckdb_impl::{ DuckDBRepo, DUCKDB_REPO };

#[async_trait::async_trait]
pub trait Repository: UserRepo + Send + Sync {
    async fn conn() -> Self where Self: Sized;
    async fn clone(&self) -> Self where Self: Sized;
    
}