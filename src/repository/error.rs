use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("DuckDB backend Error: {0}")]
    DuckDBError(#[from] duckdb::Error),
    #[error("Invalid Config: {0}")]
    InvalidConfig(String),
}
