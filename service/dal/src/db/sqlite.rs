use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{Connection, params};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::info;

use super::{ExamSpaceMemberRepo, ExamSpaceMemberRow, ExamSpaceRepo, ExamSpaceRow, UserRepo, UserRow};

/// Snowflake-like ID generator (epoch: 2024-01-01T00:00:00Z)
static LAST_MS: AtomicU64 = AtomicU64::new(0);
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const EPOCH: u64 = 1704067200000;

fn gen_token() -> String {
    let id = loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let ts = now.saturating_sub(EPOCH);

        let last = LAST_MS.load(Ordering::Relaxed);
        if ts == last {
            let seq = SEQUENCE.fetch_add(1, Ordering::Relaxed) & 0xFFF;
            if seq != 0 {
                break (ts << 12) | seq;
            }
            // sequence exhausted, spin until next ms
            while SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - EPOCH == ts {}
        } else {
            SEQUENCE.store(0, Ordering::Relaxed);
            if LAST_MS.compare_exchange(last, ts, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                break ts << 12;
            }
        }
    };
    URL_SAFE_NO_PAD.encode(id.to_be_bytes())
}

pub struct SqliteDb {
    conn: Mutex<Connection>,
}

impl SqliteDb {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let ddl = include_str!("../../../../service-ce/sql/dal_init.sql");
        conn.execute_batch(ddl)?;
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

#[async_trait]
impl ExamSpaceRepo for SqliteDb {
    async fn create_exam_space(&self, name: &str, description: &str) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        let token = gen_token();
        conn.execute(
            "INSERT INTO exam_spaces (token, name, description) VALUES (?1, ?2, ?3)",
            params![token, name, description],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    async fn update_exam_space(&self, id: i64, name: &str, description: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE exam_spaces SET name = ?1, description = ?2, last_updated_at = datetime('now') WHERE id = ?3",
                params![name, description, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("exam space not found".to_string());
        }
        Ok(())
    }

    async fn delete_exam_space(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM exam_space_members WHERE exam_space_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM exam_spaces WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn find_exam_spaces_by_user(&self, user_id: i64) -> Result<Vec<ExamSpaceRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT es.id, es.token, es.name, es.description, es.last_updated_at,
                        (SELECT COUNT(*) FROM exam_space_members WHERE exam_space_id = es.id) as member_count
                 FROM exam_spaces es
                 INNER JOIN exam_space_members esm ON es.id = esm.exam_space_id
                 WHERE esm.user_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![user_id], |row| {
                Ok(ExamSpaceRow {
                    id: row.get(0)?,
                    token: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    last_updated_at: row.get(4)?,
                    member_count: row.get(5)?,
                    paper_count: 0,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    async fn find_exam_space_by_id(&self, id: i64) -> Result<Option<ExamSpaceRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, token, name, description, last_updated_at, (SELECT COUNT(*) FROM exam_space_members WHERE exam_space_id = id) as member_count FROM exam_spaces WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![id], |row| {
            Ok(ExamSpaceRow {
                id: row.get(0)?,
                token: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                last_updated_at: row.get(4)?,
                member_count: row.get(5)?,
                paper_count: 0,
            })
        });

        match result {
            Ok(space) => Ok(Some(space)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[async_trait]
impl ExamSpaceMemberRepo for SqliteDb {
    async fn add_member(&self, user_id: i64, exam_space_id: i64, role: &str) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO exam_space_members (user_id, exam_space_id, role) VALUES (?1, ?2, ?3)",
            params![user_id, exam_space_id, role],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                "member already exists".to_string()
            } else {
                e.to_string()
            }
        })?;
        Ok(conn.last_insert_rowid())
    }

    async fn update_member_role(&self, id: i64, role: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE exam_space_members SET role = ?1 WHERE id = ?2",
                params![role, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("member not found".to_string());
        }
        Ok(())
    }

    async fn remove_member(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM exam_space_members WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn find_members_by_space(&self, exam_space_id: i64) -> Result<Vec<ExamSpaceMemberRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, exam_space_id, role, last_accessed_at
                 FROM exam_space_members WHERE exam_space_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![exam_space_id], |row| {
                Ok(ExamSpaceMemberRow {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    exam_space_id: row.get(2)?,
                    role: row.get(3)?,
                    last_accessed_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }

    async fn find_member(&self, user_id: i64, exam_space_id: i64) -> Result<Option<ExamSpaceMemberRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, exam_space_id, role, last_accessed_at
                 FROM exam_space_members WHERE user_id = ?1 AND exam_space_id = ?2",
            )
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![user_id, exam_space_id], |row| {
            Ok(ExamSpaceMemberRow {
                id: row.get(0)?,
                user_id: row.get(1)?,
                exam_space_id: row.get(2)?,
                role: row.get(3)?,
                last_accessed_at: row.get(4)?,
            })
        });

        match result {
            Ok(member) => Ok(Some(member)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}