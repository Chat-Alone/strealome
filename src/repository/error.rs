use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[cfg(feature = "repo_duckdb")]#[error("DuckDB backend Error: {0}")]
    DuckDBError(#[from] duckdb::Error),
    #[cfg(feature = "repo_sqlite")]#[error("Sqlite backend Error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Invalid Config: {0}")]
    InvalidConfig(String),
}
