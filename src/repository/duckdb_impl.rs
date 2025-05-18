use std::path::Path;
use std::sync::{ Arc, LazyLock };
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::{sync::Mutex, time };
use duckdb::{Connection, Result as DuckDBResult, Error as DuckDBError, Row, params};
use futures::StreamExt;
use crate::DATABASE_PATH;
use crate::model::UserModel;
use crate::repository::crud::CRUD;
use crate::repository::user::UserRepo;

static SCHEMA_NAME: &str = "strealome";
static DUCKDB_REPO: LazyLock<DuckDBRepo> = LazyLock::new(|| DuckDBRepo::init(DATABASE_PATH).unwrap());

pub struct DuckDBRepo {
    conn: Arc<Mutex<Connection>>,
}

pub async fn duckdb() -> Box<dyn UserRepo> {
    Box::new(DUCKDB_REPO.clone().await)
}

impl DuckDBRepo {
    fn new(conn: Connection) -> Self {
        Self { conn: Arc::new(Mutex::new(conn)) }
    }

    pub async fn clone(&self) -> Self {
        let get_conn = time::timeout(Duration::from_millis(100), self.conn.lock());
        if let Ok(conn) = get_conn.await {
            if let Ok(new_conn) = conn.try_clone() {
                return Self::new(new_conn);
            }
        }
        Self { conn: self.conn.clone() }
    }

    pub fn init(path: &str) -> DuckDBResult<Self> {
        let conn = Connection::open(path)?;
        let ret = Self::new(conn);

        ret.init_schema()?;
        ret.init_user_table()?;
        
        Ok(ret)
    }
    
    fn init_schema(&self) -> DuckDBResult<()> {
        self.conn.try_lock().unwrap().execute(
            &format!("CREATE SCHEMA IF NOT EXISTS {SCHEMA_NAME}"),[]
        ).map(|_| ())
    }
    
    fn init_user_table(&self) -> DuckDBResult<()> {
        let conn = self.conn.try_lock().unwrap();
        
        conn.execute(&format!(
            "CREATE SEQUENCE IF NOT EXISTS {SCHEMA_NAME}.users_id_seq START 1;"),[]
        )?;
        
        conn.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {SCHEMA_NAME}.users (
                id          INTEGER         PRIMARY KEY DEFAULT     nextval('{SCHEMA_NAME}.users_id_seq'),
                name        TEXT            NOT NULL,
                password    TEXT            NOT NULL,
                created_at  TIMESTAMPTZ     NOT NULL    DEFAULT     NOW()
            )"),[]
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
        let mut stmt = conn.prepare(&format!("SELECT * FROM {SCHEMA_NAME}.users WHERE id = ?")).unwrap();
        stmt.query_row(&[&id], |row| { row.try_into() }).ok()
    }

    async fn find_all(&self) -> Vec<UserModel> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(&format!("SELECT * FROM {SCHEMA_NAME}.users")).unwrap();
        if let Ok(iter) = stmt.query_map([], |row| { row.try_into() }) {
            iter.filter_map(|row| row.ok()).collect()
        } else {
            vec![]
        }
    }

    async fn create(&self, user: UserModel) -> UserModel {
        let conn = self.conn.lock().await;
        let _res = conn.execute(
            &format!("INSERT INTO {SCHEMA_NAME}.users (name, password) VALUES (?, ?)"),
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

impl UserRepo for DuckDBRepo {}