mod controller;
mod model;
mod repository;
mod service;
mod signal;

use std::sync::{Arc, LazyLock};
use repository::{ DuckDBRepo, RepoConfig };
use crate::repository::Repository;

use repository::DUCKDB_REPO as REPO;

static REPO_CFG: RepoConfig = RepoConfig {
    url: Some("res/duck.db"),
    schema: Some("strealome"),
    username: None,
    password: None,
    database: None,
};


#[tokio::main]
async fn main() -> () {
    let repo = DuckDBRepo::conn().await;
}
