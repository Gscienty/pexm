pub mod sqlite;

use async_trait::async_trait;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, String>;
    async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRow>, String>;
    async fn find_user_by_id(&self, id: i64) -> Result<Option<UserRow>, String>;
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
    async fn find_exam_spaces_by_user(
        &self,
        user_id: i64,
        after_token: Option<&str>,
        first: Option<i64>,
    ) -> Result<Vec<ExamSpaceRow>, String>;
    async fn find_exam_space_by_id(&self, id: i64) -> Result<Option<ExamSpaceRow>, String>;
    async fn find_exam_space_by_token(&self, token: &str) -> Result<Option<ExamSpaceRow>, String>;
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

#[async_trait]
pub trait DirectoryRepo: Send + Sync {
    async fn create_directory(&self, exam_space_id: i64, parent_id: Option<i64>, name: &str) -> Result<i64, String>;
    async fn update_directory(&self, id: i64, name: &str) -> Result<(), String>;
    async fn delete_directory(&self, id: i64) -> Result<(), String>;
    async fn move_directory(&self, id: i64, new_parent_id: Option<i64>) -> Result<(), String>;
    async fn find_subdirectories(&self, exam_space_id: i64, parent_id: Option<i64>) -> Result<Vec<DirectoryRow>, String>;
    async fn find_directory_by_id(&self, id: i64) -> Result<Option<DirectoryRow>, String>;
    async fn get_breadcrumb(&self, directory_id: i64) -> Result<Vec<BreadcrumbItem>, String>;
}

pub struct DirectoryRow {
    pub id: i64,
    pub exam_space_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct BreadcrumbItem {
    pub id: i64,
    pub name: String,
}

#[async_trait]
pub trait PaperRepo: Send + Sync {
    async fn create_paper(&self, exam_space_id: i64, directory_id: Option<i64>, title: &str) -> Result<i64, String>;
    async fn update_paper(&self, id: i64, title: &str) -> Result<(), String>;
    async fn delete_paper(&self, id: i64) -> Result<(), String>;
    async fn move_paper(&self, id: i64, new_directory_id: Option<i64>) -> Result<(), String>;
    async fn find_papers_by_directory(&self, exam_space_id: i64, directory_id: Option<i64>) -> Result<Vec<PaperRow>, String>;
    async fn find_paper_by_id(&self, id: i64) -> Result<Option<PaperRow>, String>;
    async fn count_papers_by_space(&self, exam_space_id: i64) -> Result<i64, String>;
    async fn store_paper_content(&self, id: i64, content: &[u8]) -> Result<(), String>;
    async fn load_paper_content(&self, id: i64) -> Result<Option<Vec<u8>>, String>;
    async fn set_paper_template(&self, id: i64, template_id: Option<i64>, template_config: Option<&[u8]>) -> Result<(), String>;
}

pub struct PaperRow {
    pub id: i64,
    pub exam_space_id: i64,
    pub directory_id: Option<i64>,
    pub title: String,
    pub content: Option<Vec<u8>>,
    pub template_id: Option<i64>,
    pub template_config: Option<Vec<u8>>,
    pub created_at: String,
    pub updated_at: String,
}

#[async_trait]
pub trait PaperTemplateRepo: Send + Sync {
    async fn create_paper_template(&self, name: &str, description: &str) -> Result<i64, String>;
    async fn update_paper_template(&self, id: i64, name: &str, description: &str, preview: Option<&[u8]>) -> Result<(), String>;
    async fn delete_paper_template(&self, id: i64) -> Result<(), String>;
    async fn find_paper_template_by_id(&self, id: i64) -> Result<Option<PaperTemplateRow>, String>;
    async fn find_all_paper_templates(&self) -> Result<Vec<PaperTemplateRow>, String>;
    async fn store_paper_template_content(&self, id: i64, content: &[u8]) -> Result<(), String>;
    async fn load_paper_template_content(&self, id: i64) -> Result<Option<Vec<u8>>, String>;
}

pub struct PaperTemplateRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub preview: Option<Vec<u8>>,
    pub created_at: String,
    pub updated_at: String,
}

pub type DbPool = std::sync::Arc<dyn UserRepo>;