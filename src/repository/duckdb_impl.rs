use std::sync::{ Arc, LazyLock };
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::{sync::Mutex, time };
use duckdb::{Connection, Result as DuckDBResult, Error as DuckDBError, Row, params};

use crate::REPO_CFG;
use crate::model::UserModel;

use super::{RepoConfig, Repository, Error};
use super::crud::CRUD;
use super::user::UserRepo;

pub static DUCKDB_REPO: LazyLock<DuckDBRepo> =
    LazyLock::new(|| DuckDBRepo::init(&REPO_CFG).expect("Failed to init") );

#[derive(Debug)]
pub struct DuckDBRepo {
    schema_name: String,
    conn: Arc<Mutex<Connection>>,
}

#[async_trait]
impl Repository for DuckDBRepo {
    async fn conn() -> Self {
        DUCKDB_REPO.clone().await
    }
    
    async fn clone(&self) -> Self {
        let get_conn = time::timeout(Duration::from_millis(100), self.conn.lock());
        if let Ok(conn) = get_conn.await {
            if let Ok(new_conn) = conn.try_clone() {
                return Self::new(new_conn, self.schema_name.clone());
            }
        }
        Self {
            schema_name: self.schema_name.clone(),
            conn: self.conn.clone()
        }
    }
}


impl DuckDBRepo {
    fn new(conn: Connection, schema_name: String) -> Self {
        Self {
            schema_name,
            conn: Arc::new(Mutex::new(conn))
        }
    }

    fn init(cfg: &RepoConfig) -> Result<Self, Error> {
        if let RepoConfig { url: Some(url), schema: Some(schema), .. } = cfg {
            let conn = Connection::open(url)?;
            let ret = Self::new(conn, schema.to_string());

            ret.init_schema()?;
            ret.init_user_table()?;

            Ok(ret)
        } else {
            Err(Error::InvalidConfig(format!("{cfg:?}")))
        }

    }
    
    fn init_schema(&self) -> DuckDBResult<()> {
        self.conn.try_lock().unwrap().execute(
            &format!("CREATE SCHEMA IF NOT EXISTS {}", self.schema_name),[]
        ).map(|_| ())
    }
    
    fn init_user_table(&self) -> DuckDBResult<()> {
        let conn = self.conn.try_lock().unwrap();
        
        conn.execute(&format!(
            "CREATE SEQUENCE IF NOT EXISTS {}.users_id_seq START 1;", self.schema_name),[]
        )?;
        
        conn.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {}.users (
                id          INTEGER         PRIMARY KEY DEFAULT     nextval('{}.users_id_seq'),
                name        TEXT            NOT NULL,
                password    TEXT            NOT NULL,
                created_at  TIMESTAMPTZ     NOT NULL    DEFAULT     NOW()
            )", self.schema_name, self.schema_name),[]
        ).map(|_| ())
    }
}

impl<'a> TryFrom<&Row<'a>> for UserModel {
    type Error = DuckDBError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            password: row.get(2)?,
            created_at: DateTime::from_timestamp_millis(row.get(3)?).unwrap_or(Utc::now()),
        })
    }
}

#[async_trait::async_trait]
impl CRUD for DuckDBRepo {
    type Target = UserModel;

    async fn find_by_id(&self, id: i32) -> Option<UserModel> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(&format!("SELECT * FROM {}.users WHERE id = ?", self.schema_name)).unwrap();
        stmt.query_row(&[&id], |row| { row.try_into() }).ok()
    }

    async fn find_all(&self) -> Vec<UserModel> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(&format!("SELECT * FROM {}.users", self.schema_name)).unwrap();
        if let Ok(iter) = stmt.query_map([], |row| { row.try_into() }) {
            iter.filter_map(|row| row.ok()).collect()
        } else {
            vec![]
        }
    }

    async fn create(&self, user: UserModel) -> UserModel {
        let conn = self.conn.lock().await;
        let _res = conn.execute(
            &format!("INSERT INTO {}.users (name, password) VALUES (?, ?)", self.schema_name),
            params![&user.name, &user.password]
        ).unwrap();
        
        user
    }

    async fn update(&self, user: UserModel) -> UserModel {
        todo!()
    }

    async fn delete(&self, id: i32) -> bool {
        todo!()
    }
}

#[async_trait::async_trait]
impl UserRepo for DuckDBRepo {
    async fn find_by_name(&self, name: &str) -> Result<Option<UserModel>, Error> {
        let conn = self.conn.lock().await;
        let res = conn.query_row(
            &format!("SELECT * FROM {}.users WHERE name = ?", self.schema_name),
            &[name], |row| { UserModel::try_from(row) }
        );
        match res {
            Ok(user) => Ok(Some(user)),
            Err(e) => {
                match e {
                    DuckDBError::InvalidColumnType(_,_,_)   |
                    DuckDBError::InvalidColumnIndex(_)      |
                    DuckDBError::InvalidColumnName(_) => { Ok(None) },
                    _ => { Err(e.into()) }
                }
            }
        }
    }
}