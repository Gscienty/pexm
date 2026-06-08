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
    pub username: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlPageInfo {
    pub has_next_page: bool,
    pub end_cursor: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlExamSpaceEdge {
    pub node: GqlExamSpace,
    pub cursor: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlExamSpaceConnection {
    pub edges: Vec<GqlExamSpaceEdge>,
    pub page_info: GqlPageInfo,
}

/// Used when `token` is provided: returns a single space (not wrapped in connection)
#[derive(SimpleObject, Clone)]
pub struct GqlExamSpacePayload {
    pub space: Option<GqlExamSpace>,
}

/// Union: either a paginated connection, or a single space payload
#[derive(async_graphql::Union, Clone)]
pub enum GqlExamSpacesResult {
    Connection(GqlExamSpaceConnection),
    Payload(GqlExamSpacePayload),
}

// --- Directory & Paper ---

#[derive(SimpleObject, Clone)]
pub struct GqlDirectory {
    pub id: i64,
    pub exam_space_id: i64,
    pub parent_id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlPaper {
    pub id: i64,
    pub exam_space_id: i64,
    pub directory_id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlDirectoryChildren {
    pub directories: Vec<GqlDirectory>,
    pub papers: Vec<GqlPaper>,
}

#[derive(SimpleObject, Clone)]
pub struct GqlBreadcrumbItem {
    pub id: i64,
    pub name: String,
}

#[derive(SimpleObject, Clone)]
pub struct GqlPaperBreadcrumb {
    pub directory_items: Vec<GqlBreadcrumbItem>,
    pub paper: GqlPaper,
}