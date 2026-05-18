use async_trait::async_trait;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;
use tracing::info;

use super::{UserRepo, UserRow};

pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                username    TEXT NOT NULL,
                email       TEXT NOT NULL UNIQUE,
                password    TEXT NOT NULL
            );",
        )?;
        info!("SQLite initialized: {}", path);
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[async_trait]
impl UserRepo for SqliteDb {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO users (username, email, password) VALUES (?1, ?2, ?3)",
            params![username, email, password_hash],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                "email already exists".to_string()
            } else {
                e.to_string()
            }
        })?;

        Ok(conn.last_insert_rowid())
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, username, password FROM users WHERE email = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params![email], |row| {
                Ok(UserRow {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                })
            });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}