use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone)]
pub struct GqlExamSpace {
    pub id: i64,
    pub token: String,
    pub name: String,
    pub description: String,
    pub last_updated_at: String,
    pub member_count: i64,
    pub paper_count: i64,
}

#[derive(SimpleObject, Clone)]
pub struct GqlExamSpaceMember {
    pub id: i64,
    pub user_id: i64,
    pub exam_space_id: i64,
    pub role: String,
    pub last_accessed_at: Option<String>,
}