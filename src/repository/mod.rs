mod user;
mod duckdb_impl;
mod crud;

pub use crud::CRUD;
pub use duckdb_impl::{ DuckDBRepo, duckdb };
