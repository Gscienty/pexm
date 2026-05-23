use async_graphql::{Context, Object};

use auth_lib::claim::Claim;

use crate::rpcclient::dal::{
    dal_service_client::DalServiceClient, ListExamSpacesRequest,
};

use super::types::*;

pub struct Query;

#[Object]
impl Query {
    async fn my_exam_spaces(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<GqlExamSpace>> {
        let claim = ctx
            .data::<Claim>()
            .map_err(|_| "unauthorized")?;

        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .list_exam_spaces_by_user(ListExamSpacesRequest {
                user_id: claim.get_subject() as i64,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();

        Ok(response
            .spaces
            .into_iter()
            .map(|s| GqlExamSpace {
                id: s.id,
                token: s.token,
                name: s.name,
                description: s.description,
                last_updated_at: s.last_updated_at,
                member_count: s.member_count,
                paper_count: s.paper_count,
            })
            .collect())
    }

    async fn exam_space_members(
        &self,
        ctx: &Context<'_>,
        exam_space_id: i64,
    ) -> async_graphql::Result<Vec<GqlExamSpaceMember>> {
        let mut client = ctx
            .data::<DalServiceClient<tonic::transport::Channel>>()
            .map_err(|_| "dal client unavailable")?
            .clone();

        let response = client
            .list_exam_space_members(
                crate::rpcclient::dal::ListExamSpaceMembersRequest { exam_space_id },
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
            })
            .collect())
    }
}