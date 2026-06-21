use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{Connection, params};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::info;

use super::{ExamSpaceMemberRepo, ExamSpaceMemberRow, ExamSpaceRepo, ExamSpaceRow, UserRepo, UserRow, DirectoryRepo, DirectoryRow, BreadcrumbItem, PaperRepo, PaperRow};

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

    async fn find_user_by_id(&self, id: i64) -> Result<Option<UserRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, username, password FROM users WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params![id], |row| {
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

    async fn find_exam_spaces_by_user(
        &self,
        user_id: i64,
        after_token: Option<&str>,
        first: Option<i64>,
    ) -> Result<Vec<ExamSpaceRow>, String> {
        let conn = self.conn.lock().await;
        let limit = first.unwrap_or(20).max(1).min(100);
        let fetch_one_more = limit + 1;

        let base_sql = "SELECT es.id, es.token, es.name, es.description, es.last_updated_at,
                        (SELECT COUNT(*) FROM exam_space_members WHERE exam_space_id = es.id) as member_count,
                        es.paper_count
                 FROM exam_spaces es
                 INNER JOIN exam_space_members esm ON es.id = esm.exam_space_id
                 WHERE esm.user_id = ?1";

        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match after_token {
            Some(token) => (
                format!("{} AND es.token > ?2", base_sql),
                vec![Box::new(user_id), Box::new(token.to_string())],
            ),
            None => (base_sql.to_string(), vec![Box::new(user_id)]),
        };

        let sql_with_order = format!("{} ORDER BY es.token ASC LIMIT {}", sql, fetch_one_more);

        let mut stmt = conn.prepare(&sql_with_order).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())), |row| {
                Ok(ExamSpaceRow {
                    id: row.get(0)?,
                    token: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    last_updated_at: row.get(4)?,
                    member_count: row.get(5)?,
                    paper_count: row.get(6)?,
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
            .prepare("SELECT id, token, name, description, last_updated_at, (SELECT COUNT(*) FROM exam_space_members WHERE exam_space_id = exam_spaces.id) as member_count, paper_count FROM exam_spaces WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![id], |row| {
            Ok(ExamSpaceRow {
                id: row.get(0)?,
                token: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                last_updated_at: row.get(4)?,
                member_count: row.get(5)?,
                paper_count: row.get(6)?,
            })
        });

        match result {
            Ok(space) => Ok(Some(space)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn find_exam_space_by_token(&self, token: &str) -> Result<Option<ExamSpaceRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, token, name, description, last_updated_at, (SELECT COUNT(*) FROM exam_space_members WHERE exam_space_id = exam_spaces.id) as member_count, paper_count FROM exam_spaces WHERE token = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![token], |row| {
            Ok(ExamSpaceRow {
                id: row.get(0)?,
                token: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                last_updated_at: row.get(4)?,
                member_count: row.get(5)?,
                paper_count: row.get(6)?,
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

#[async_trait]
impl DirectoryRepo for SqliteDb {
    async fn create_directory(&self, exam_space_id: i64, parent_id: Option<i64>, name: &str) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO directories (exam_space_id, parent_id, name) VALUES (?1, ?2, ?3)",
            params![exam_space_id, parent_id, name],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    async fn update_directory(&self, id: i64, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE directories SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![name, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("directory not found".to_string());
        }
        Ok(())
    }

    async fn delete_directory(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        // Move papers in this directory to root (set directory_id to NULL)
        conn.execute("UPDATE papers SET directory_id = NULL WHERE directory_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        // Move sub-directories to parent of this directory
        let parent_id: Option<i64> = conn
            .query_row("SELECT parent_id FROM directories WHERE id = ?1", params![id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute("UPDATE directories SET parent_id = ?1 WHERE parent_id = ?2", params![parent_id, id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM directories WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn move_directory(&self, id: i64, new_parent_id: Option<i64>) -> Result<(), String> {
        let conn = self.conn.lock().await;
        // Prevent moving into own subtree
        if let Some(pid) = new_parent_id {
            let mut check_id = pid;
            loop {
                if check_id == id {
                    return Err("cannot move directory into its own subtree".to_string());
                }
                let parent: Option<i64> = conn
                    .query_row("SELECT parent_id FROM directories WHERE id = ?1", params![check_id], |row| row.get(0))
                    .ok()
                    .flatten();
                match parent {
                    Some(p) => check_id = p,
                    None => break,
                }
            }
        }
        let affected = conn
            .execute(
                "UPDATE directories SET parent_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![new_parent_id, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("directory not found".to_string());
        }
        Ok(())
    }

    async fn find_subdirectories(&self, exam_space_id: i64, parent_id: Option<i64>) -> Result<Vec<DirectoryRow>, String> {
        let conn = self.conn.lock().await;
        let sql = match parent_id {
            Some(_) => "SELECT id, exam_space_id, parent_id, name, created_at, updated_at FROM directories WHERE exam_space_id = ?1 AND parent_id = ?2 ORDER BY name ASC",
            None => "SELECT id, exam_space_id, parent_id, name, created_at, updated_at FROM directories WHERE exam_space_id = ?1 AND parent_id IS NULL ORDER BY name ASC",
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = match parent_id {
            Some(pid) => stmt
                .query_map(params![exam_space_id, pid], |row| {
                    Ok(DirectoryRow {
                        id: row.get(0)?,
                        exam_space_id: row.get(1)?,
                        parent_id: row.get(2)?,
                        name: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
            None => stmt
                .query_map(params![exam_space_id], |row| {
                    Ok(DirectoryRow {
                        id: row.get(0)?,
                        exam_space_id: row.get(1)?,
                        parent_id: row.get(2)?,
                        name: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
        };
        Ok(rows)
    }

    async fn find_directory_by_id(&self, id: i64) -> Result<Option<DirectoryRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, exam_space_id, parent_id, name, created_at, updated_at FROM directories WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let result = stmt.query_row(params![id], |row| {
            Ok(DirectoryRow {
                id: row.get(0)?,
                exam_space_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        });
        match result {
            Ok(dir) => Ok(Some(dir)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_breadcrumb(&self, directory_id: i64) -> Result<Vec<BreadcrumbItem>, String> {
        let conn = self.conn.lock().await;
        let mut items = Vec::new();
        let mut current_id = Some(directory_id);
        while let Some(cid) = current_id {
            let result: Result<(i64, String, Option<i64>), _> = conn
                .query_row(
                    "SELECT id, name, parent_id FROM directories WHERE id = ?1",
                    params![cid],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
            match result {
                Ok((id, name, parent_id)) => {
                    items.push(BreadcrumbItem { id, name });
                    current_id = parent_id;
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => break,
                Err(e) => return Err(e.to_string()),
            }
        }
        items.reverse();
        Ok(items)
    }
}

#[async_trait]
impl PaperRepo for SqliteDb {
    async fn create_paper(&self, exam_space_id: i64, directory_id: Option<i64>, title: &str) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO papers (exam_space_id, directory_id, title) VALUES (?1, ?2, ?3)",
            params![exam_space_id, directory_id, title],
        )
        .map_err(|e| e.to_string())?;
        // Update space last_updated_at and increment paper_count
        conn.execute(
            "UPDATE exam_spaces SET last_updated_at = datetime('now'), paper_count = paper_count + 1 WHERE id = ?1",
            params![exam_space_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    async fn update_paper(&self, id: i64, title: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE papers SET title = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![title, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("paper not found".to_string());
        }
        Ok(())
    }

    async fn delete_paper(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        // Get exam_space_id before deleting
        let exam_space_id: i64 = conn
            .query_row("SELECT exam_space_id FROM papers WHERE id = ?1", params![id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM papers WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE exam_spaces SET paper_count = paper_count - 1 WHERE id = ?1 AND paper_count > 0",
            params![exam_space_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn move_paper(&self, id: i64, new_directory_id: Option<i64>) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE papers SET directory_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![new_directory_id, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("paper not found".to_string());
        }
        Ok(())
    }

    async fn find_papers_by_directory(&self, exam_space_id: i64, directory_id: Option<i64>) -> Result<Vec<PaperRow>, String> {
        let conn = self.conn.lock().await;
        let sql = match directory_id {
            Some(_) => "SELECT id, exam_space_id, directory_id, title, created_at, updated_at FROM papers WHERE exam_space_id = ?1 AND directory_id = ?2 ORDER BY title ASC",
            None => "SELECT id, exam_space_id, directory_id, title, created_at, updated_at FROM papers WHERE exam_space_id = ?1 AND directory_id IS NULL ORDER BY title ASC",
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = match directory_id {
            Some(did) => stmt
                .query_map(params![exam_space_id, did], |row| {
                    Ok(PaperRow {
                        id: row.get(0)?,
                        exam_space_id: row.get(1)?,
                        directory_id: row.get(2)?,
                        title: row.get(3)?,
                        content: None,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
            None => stmt
                .query_map(params![exam_space_id], |row| {
                    Ok(PaperRow {
                        id: row.get(0)?,
                        exam_space_id: row.get(1)?,
                        directory_id: row.get(2)?,
                        title: row.get(3)?,
                        content: None,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?,
        };
        Ok(rows)
    }

    async fn find_paper_by_id(&self, id: i64) -> Result<Option<PaperRow>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, exam_space_id, directory_id, title, created_at, updated_at FROM papers WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let result = stmt.query_row(params![id], |row| {
            Ok(PaperRow {
                id: row.get(0)?,
                exam_space_id: row.get(1)?,
                directory_id: row.get(2)?,
                title: row.get(3)?,
                content: None,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        });
        match result {
            Ok(paper) => Ok(Some(paper)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn count_papers_by_space(&self, exam_space_id: i64) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM papers WHERE exam_space_id = ?1",
                params![exam_space_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(count)
    }

    async fn store_paper_content(&self, id: i64, content: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let affected = conn
            .execute(
                "UPDATE papers SET content = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![content, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("paper not found".to_string());
        }
        Ok(())
    }

    async fn load_paper_content(&self, id: i64) -> Result<Option<Vec<u8>>, String> {
        let conn = self.conn.lock().await;
        let result = conn
            .query_row(
                "SELECT content FROM papers WHERE id = ?1",
                params![id],
                |row| row.get::<_, Option<Vec<u8>>>(0),
            );
        match result {
            Ok(content) => Ok(content),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}