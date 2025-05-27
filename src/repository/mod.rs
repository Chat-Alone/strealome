mod user;
mod crud;
mod config;
mod error;
#[cfg(feature = "repo_duckdb")]
mod duckdb_impl;
#[cfg(feature = "repo_sqlite")]
mod sqlite_impl;

pub use crud::CRUD;
pub use error::Error;
pub use config::RepoConfig;
pub use user::UserRepo;
#[cfg(feature = "repo_duckdb")]
pub use duckdb_impl::{ DuckDBRepo as Repo, DUCKDB_REPO as REPO };
#[cfg(feature = "repo_sqlite")]
pub use sqlite_impl::{ SqliteRepo as Repo, SQLITE_REPO as REPO };

#[async_trait::async_trait]
pub trait Repository: UserRepo + Send + Sync {
    async fn conn() -> Self where Self: Sized;
    async fn clone(&self) -> Self where Self: Sized;
    
}