use async_graphql::{Context, Object};
use base64::Engine;

use auth_lib::claim::Claim;

use crate::rpcclient::dal::{
    dal_service_client::DalServiceClient, GetExamSpaceByTokenRequest, ListExamSpacesRequest,
    ListDirectoryChildrenRequest, GetDirectoryBreadcrumbRequest,
    CreateDirectoryRequest, CreatePaperRequest, DeleteDirectoryRequest, DeletePaperRequest,
    UpdateDirectoryRequest, UpdatePaperRequest, MoveDirectoryRequest, MovePaperRequest,
    GetPaperBreadcrumbRequest, UpdateExamSpaceRequest, DeleteExamSpaceRequest,
};

use super::types::*;

fn paper_to_gql(p: crate::rpcclient::dal::Paper) -> GqlPaper {
    GqlPaper {
        id: p.id,
        exam_space_id: p.exam_space_id,
        directory_id: p.directory_id,
        title: p.title,
        created_at: p.created_at,
        updated_at: p.updated_at,
        template_id: p.template_id,
        template_config: base64::engine::general_purpose::STANDARD.encode(&p.template_config),
    }
}

pub struct Query;

#[Object]
impl Query {
    /// exam_spaces(token, first, after)
    /// - token provided: returns single space (GqlExamSpacesResult::Payload)
    /// - token omitted: returns paginated list (GqlExamSpacesResult::Connection)
    async fn exam_spaces(
        &self,
        ctx: &Context<'_>,
        token: Option<String>,
        first: Option<i64>,
        after: Option<String>,
    ) -> async_graphql::Result<GqlExamSpacesResult> {
        let claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        // Single space lookup when token is provided
        if let Some(ref tok) = token {
            let response = client
                .get_exam_space_by_token(GetExamSpaceByTokenRequest {
                    token: tok.clone(),
                })
                .await
                .map_err(|e| e.to_string())?
                .into_inner();

            return Ok(GqlExamSpacesResult::Payload(GqlExamSpacePayload {
                space: response.space.map(|s| GqlExamSpace {
                    id: s.id,
                    token: s.token,
                    name: s.name,
                    description: s.description,
                    last_updated_at: s.last_updated_at,
                    member_count: s.member_count,
                    paper_count: s.paper_count,
                }),
            }));
        }

        // Paginated list
        let response = client
            .list_exam_spaces_by_user(ListExamSpacesRequest {
                user_id: claim.get_subject() as i64,
                after_token: after.clone().unwrap_or_default(),
                first: first.unwrap_or(20),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        let edges = response
            .spaces
            .into_iter()
            .map(|s| {
                let node = GqlExamSpace {
                    id: s.id,
                    token: s.token.clone(),
                    name: s.name,
                    description: s.description,
                    last_updated_at: s.last_updated_at,
                    member_count: s.member_count,
                    paper_count: s.paper_count,
                };
                GqlExamSpaceEdge {
                    node,
                    cursor: s.token,
                }
            })
            .collect();

        Ok(GqlExamSpacesResult::Connection(GqlExamSpaceConnection {
            edges,
            page_info: GqlPageInfo {
                has_next_page: response.has_next_page,
                end_cursor: response.end_cursor,
            },
        }))
    }

    async fn exam_space_members(
        &self,
        ctx: &Context<'_>,
        exam_space_id: String,
    ) -> async_graphql::Result<Vec<GqlExamSpaceMember>> {
        let id: i64 = exam_space_id.parse().map_err(|_| "invalid exam_space_id")?;
        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .list_exam_space_members(
                crate::rpcclient::dal::ListExamSpaceMembersRequest { exam_space_id: id },
            )
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        Ok(response
            .members
            .into_iter()
            .map(|m| GqlExamSpaceMember {
                id: m.id,
                user_id: m.user_id,
                exam_space_id: m.exam_space_id,
                role: m.role,
                last_accessed_at: if m.last_accessed_at.is_empty() {
                    None
                } else {
                    Some(m.last_accessed_at)
                },
                username: m.username,
            })
            .collect())
    }

    /// List children (sub-directories + papers) of a directory in an exam space.
    /// parent_id = 0 means root directory.
    async fn directory_children(
        &self,
        ctx: &Context<'_>,
        exam_space_id: i64,
        parent_id: Option<i64>,
    ) -> async_graphql::Result<GqlDirectoryChildren> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .list_directory_children(ListDirectoryChildrenRequest {
                exam_space_id,
                parent_id: parent_id.unwrap_or(0),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        Ok(GqlDirectoryChildren {
            directories: response.directories.into_iter().map(|d| GqlDirectory {
                id: d.id,
                exam_space_id: d.exam_space_id,
                parent_id: d.parent_id,
                name: d.name,
                created_at: d.created_at,
                updated_at: d.updated_at,
            }).collect(),
            papers: response.papers.into_iter().map(paper_to_gql).collect(),
        })
    }

    /// Get breadcrumb path from root to a directory
    async fn directory_breadcrumb(
        &self,
        ctx: &Context<'_>,
        directory_id: i64,
    ) -> async_graphql::Result<Vec<GqlBreadcrumbItem>> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .get_directory_breadcrumb(GetDirectoryBreadcrumbRequest {
                directory_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        Ok(response.items.into_iter().map(|i| GqlBreadcrumbItem {
            id: i.id,
            name: i.name,
        }).collect())
    }

    /// Get breadcrumb path from root to a paper
    async fn paper_breadcrumb(
        &self,
        ctx: &Context<'_>,
        paper_id: i64,
    ) -> async_graphql::Result<GqlPaperBreadcrumb> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .get_paper_breadcrumb(GetPaperBreadcrumbRequest {
                paper_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        let paper = response.paper.ok_or("paper not found")?;

        Ok(GqlPaperBreadcrumb {
            directory_items: response.directory_items.into_iter().map(|i| GqlBreadcrumbItem {
                id: i.id,
                name: i.name,
            }).collect(),
            paper: paper_to_gql(paper),
        })
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_directory(
        &self,
        ctx: &Context<'_>,
        exam_space_id: i64,
        parent_id: Option<i64>,
        name: String,
    ) -> async_graphql::Result<GqlDirectory> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let create_response = client
            .create_directory(CreateDirectoryRequest {
                exam_space_id,
                parent_id: parent_id.unwrap_or(0),
                name,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        // Fetch the created directory
        let list_response = client
            .list_directory_children(ListDirectoryChildrenRequest {
                exam_space_id,
                parent_id: parent_id.unwrap_or(0),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        let dir = list_response
            .directories
            .into_iter()
            .find(|d| d.id == create_response.id)
            .ok_or("created directory not found")?;

        Ok(GqlDirectory {
            id: dir.id,
            exam_space_id: dir.exam_space_id,
            parent_id: dir.parent_id,
            name: dir.name,
            created_at: dir.created_at,
            updated_at: dir.updated_at,
        })
    }

    async fn update_directory(
        &self,
        ctx: &Context<'_>,
        id: i64,
        name: String,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .update_directory(UpdateDirectoryRequest { id, name })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn delete_directory(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .delete_directory(DeleteDirectoryRequest { id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn move_directory(
        &self,
        ctx: &Context<'_>,
        id: i64,
        new_parent_id: Option<i64>,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .move_directory(MoveDirectoryRequest {
                id,
                new_parent_id: new_parent_id.unwrap_or(0),
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn create_paper(
        &self,
        ctx: &Context<'_>,
        exam_space_id: i64,
        directory_id: Option<i64>,
        title: String,
    ) -> async_graphql::Result<GqlPaper> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let create_response = client
            .create_paper(CreatePaperRequest {
                exam_space_id,
                directory_id: directory_id.unwrap_or(0),
                title,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        // Fetch the created paper via list
        let list_response = client
            .list_directory_children(ListDirectoryChildrenRequest {
                exam_space_id,
                parent_id: directory_id.unwrap_or(0),
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        let paper = list_response
            .papers
            .into_iter()
            .find(|p| p.id == create_response.id)
            .ok_or("created paper not found")?;

        Ok(paper_to_gql(paper))
    }

    async fn update_paper(
        &self,
        ctx: &Context<'_>,
        id: i64,
        title: String,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .update_paper(UpdatePaperRequest { id, title })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn delete_paper(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .delete_paper(DeletePaperRequest { id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn move_paper(
        &self,
        ctx: &Context<'_>,
        id: i64,
        new_directory_id: Option<i64>,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .move_paper(MovePaperRequest {
                id,
                new_directory_id: new_directory_id.unwrap_or(0),
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn update_exam_space(
        &self,
        ctx: &Context<'_>,
        id: i64,
        name: String,
        description: String,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .update_exam_space(UpdateExamSpaceRequest {
                id,
                name,
                description,
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn delete_exam_space(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> async_graphql::Result<bool> {
        let _claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        client
            .delete_exam_space(DeleteExamSpaceRequest { id })
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }
}