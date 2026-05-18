pub mod sqlite;

use async_trait::async_trait;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, String>;
    async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRow>, String>;
}

pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

pub type DbPool = std::sync::Arc<dyn UserRepo>;