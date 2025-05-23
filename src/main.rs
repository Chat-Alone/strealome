mod controller;
mod model;
mod repository;
mod service;
mod signal;

use std::sync::{Arc};
use axum::serve;
use chrono::Duration;
use tokio::task::JoinHandle;
use repository::{DuckDBRepo, RepoConfig };
use crate::repository::Repository;

const USE_COOKIE: bool = true;

use repository::DUCKDB_REPO as REPO;

static REPO_CFG: RepoConfig = RepoConfig {
    url: Some("res/duck.db"),
    schema: Some("strealome"),
    username: None,
    password: None,
    database: None,
};

async fn ctrl_c_task() -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("Ctrl-C received, shutting down.");
    })
}

#[tokio::main]
async fn main() -> () {
    let repo = DuckDBRepo::conn().await;
    let mut serve_task = controller::listen(
        "127.0.0.1:3000", 
        Arc::new(repo), 
        "secret".to_string(),
        Duration::minutes(30)
    ).await;
    
    let mut ctrl_c_task = ctrl_c_task().await;
    
    tokio::select! {
        _ = &mut ctrl_c_task => {
            println!("Ctrl-C received, shutting down.");
            serve_task.abort()
        },
        res = &mut serve_task => {
            println!("Server stopped.");
            if let Err(e) = res.unwrap() {
                println!("Server error: {}", e);
            }
            ctrl_c_task.abort()
        }
    }
}
