mod controller;
mod model;
mod repository;
mod service;

use std::sync::{Arc};
use chrono::Duration;
use tokio::task::JoinHandle;
use repository::{ Repo, RepoConfig };
use crate::repository::Repository;


static REPO_CFG: RepoConfig = RepoConfig {
    url: Some("res/sqlite.db"),
    schema: Some("strealome"),
    username: None,
    password: None,
    database: None,
};

async fn ctrl_c_task() -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
    })
}

#[tokio::main]
async fn main() {
    let repo = Repo::conn().await;

    let mut serve_task = controller::listen(
        "0.0.0.0:56657",
        Arc::new(repo),
        "secret".to_string(),
        Duration::minutes(30),
        Duration::days(3),
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
