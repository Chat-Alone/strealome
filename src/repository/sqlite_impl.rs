use std::sync::{ Arc, LazyLock };
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::{sync::Mutex, time };
use rusqlite::{Connection, Result as SqliteResult, Error as SqliteError, Row, params};

use crate::REPO_CFG;
use crate::model::UserModel;

use super::{RepoConfig, Repository, Error};
use super::crud::CRUD;
use super::user::UserRepo;

pub static SQLITE_REPO: LazyLock<SqliteRepo> =
    LazyLock::new(|| SqliteRepo::init(&REPO_CFG).expect("Failed to init") );

#[derive(Debug)]
pub struct SqliteRepo {
    schema_name: String,
    conn: Arc<Mutex<Connection>>,
}

#[async_trait]
impl Repository for SqliteRepo {
    async fn conn() -> Self {
        SQLITE_REPO.clone().await
    }

    async fn clone(&self) -> Self {
        Self {
            schema_name: self.schema_name.clone(),
            conn: self.conn.clone()
        }
    }
}


impl SqliteRepo {
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

    fn init_schema(&self) -> SqliteResult<()> {
        self.conn.try_lock().unwrap().execute(
            &format!("CREATE SCHEMA IF NOT EXISTS {}", self.schema_name),[]
        ).map(|_| ())
    }

    fn init_user_table(&self) -> SqliteResult<()> {
        let conn = self.conn.try_lock().unwrap();

        conn.execute(&format!(
            "CREATE SEQUENCE IF NOT EXISTS {}.users_id_seq START 1;", self.schema_name),[]
        )?;

        conn.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {}.users (
                id          INTEGER         PRIMARY KEY DEFAULT     nextval('{}.users_id_seq'),
                name        TEXT            NOT NULL    UNIQUE,
                password    TEXT            NOT NULL,
                created_at  TIMESTAMPTZ     NOT NULL    DEFAULT     NOW()
            )", self.schema_name, self.schema_name),[]
        ).map(|_| ())
    }
}

impl<'a> TryFrom<&Row<'a>> for UserModel {
    type Error = SqliteError;

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
impl CRUD for SqliteRepo {
    type Target = UserModel;
    type Error = Error;

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

    async fn update(&self, user: UserModel) -> Result<UserModel, Error> {
        let conn = self.conn.lock().await;
        let user = conn.query_row(
            &format!("UPDATE {}.users SET name = ?, password = ? WHERE id = ? RETURNING *", self.schema_name),
            params![&user.name, &user.password, &user.id], |row| { UserModel::try_from(row) }
        )?;

        Ok(user)
    }

    async fn delete(&self, id: i32) -> bool {
        todo!()
    }
}

#[async_trait::async_trait]
impl UserRepo for SqliteRepo {
    /// Find user by name
    ///
    /// # Parameters
    /// * `name` - The name of the user to find
    ///
    /// # Returns
    /// * `Ok(Some(UserModel))` - User found, returns the user model
    /// * `Ok(None)` - User not found
    /// * `Err(Error)` - Error occurred during the query
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
                    SqliteError::QueryReturnedNoRows        |
                    SqliteError::InvalidColumnType(_,_,_)   |
                    SqliteError::InvalidColumnIndex(_)      |
                    SqliteError::InvalidColumnName(_) => { Ok(None) },
                    _ => { Err(e.into()) }
                }
            }
        }
    }
}