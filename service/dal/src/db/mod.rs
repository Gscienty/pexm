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

#[async_trait]
pub trait ExamSpaceRepo: Send + Sync {
    async fn create_exam_space(&self, name: &str, description: &str) -> Result<i64, String>;
    async fn update_exam_space(&self, id: i64, name: &str, description: &str) -> Result<(), String>;
    async fn delete_exam_space(&self, id: i64) -> Result<(), String>;
    async fn find_exam_spaces_by_user(&self, user_id: i64) -> Result<Vec<ExamSpaceRow>, String>;
    async fn find_exam_space_by_id(&self, id: i64) -> Result<Option<ExamSpaceRow>, String>;
}

pub struct ExamSpaceRow {
    pub id: i64,
    pub token: String,
    pub name: String,
    pub description: String,
    pub last_updated_at: String,
    pub member_count: i64,
    pub paper_count: i64,
}

#[async_trait]
pub trait ExamSpaceMemberRepo: Send + Sync {
    async fn add_member(&self, user_id: i64, exam_space_id: i64, role: &str) -> Result<i64, String>;
    async fn update_member_role(&self, id: i64, role: &str) -> Result<(), String>;
    async fn remove_member(&self, id: i64) -> Result<(), String>;
    async fn find_members_by_space(&self, exam_space_id: i64) -> Result<Vec<ExamSpaceMemberRow>, String>;
    async fn find_member(&self, user_id: i64, exam_space_id: i64) -> Result<Option<ExamSpaceMemberRow>, String>;
}

pub struct ExamSpaceMemberRow {
    pub id: i64,
    pub user_id: i64,
    pub exam_space_id: i64,
    pub role: String,
    pub last_accessed_at: Option<String>,
}

pub type DbPool = std::sync::Arc<dyn UserRepo>;