use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use crate::dal::{
    AddExamSpaceMemberRequest, AddExamSpaceMemberResponse, CreateExamSpaceRequest,
    CreateExamSpaceResponse, DeleteExamSpaceRequest, Empty, ExamSpace,
    ExamSpaceMember, ListExamSpaceMembersRequest, ListExamSpaceMembersResponse,
    ListExamSpacesRequest, ListExamSpacesResponse, RemoveExamSpaceMemberRequest,
    UpdateExamSpaceMemberRequest, UpdateExamSpaceRequest,
};
use crate::db::{ExamSpaceMemberRepo, ExamSpaceRepo};

#[derive(Clone)]
pub struct ExamSpaceHandler {
    exam_space_db: Arc<dyn ExamSpaceRepo>,
    member_db: Arc<dyn ExamSpaceMemberRepo>,
}

impl ExamSpaceHandler {
    pub fn new(
        exam_space_db: Arc<dyn ExamSpaceRepo>,
        member_db: Arc<dyn ExamSpaceMemberRepo>,
    ) -> Self {
        Self {
            exam_space_db,
            member_db,
        }
    }

    pub async fn create_exam_space(
        &self,
        request: Request<CreateExamSpaceRequest>,
    ) -> Result<Response<CreateExamSpaceResponse>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        let description = req.description.trim().to_string();

        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }

        let space_id = self
            .exam_space_db
            .create_exam_space(&name, &description)
            .await
            .map_err(|e| Status::internal(e))?;

        // 获取刚创建空间的 token
        let space = self
            .exam_space_db
            .find_exam_space_by_id(space_id)
            .await
            .map_err(|e| Status::internal(e))?
            .ok_or_else(|| Status::internal("created space not found"))?;

        // 创建者自动添加为 owner 成员
        self.member_db
            .add_member(req.user_id, space_id, "owner")
            .await
            .map_err(|e| {
                tracing::error!("Failed to add owner member: {}", e);
                Status::internal(e)
            })?;

        info!("Exam space created: id={}, name={}, owner={}", space_id, name, req.user_id);

        Ok(Response::new(CreateExamSpaceResponse { id: space_id, token: space.token }))
    }

    pub async fn update_exam_space(
        &self,
        request: Request<UpdateExamSpaceRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        let description = req.description.trim().to_string();

        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }

        self.exam_space_db
            .update_exam_space(req.id, &name, &description)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Exam space updated: id={}", req.id);

        Ok(Response::new(Empty {}))
    }

    pub async fn delete_exam_space(
        &self,
        request: Request<DeleteExamSpaceRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.exam_space_db
            .delete_exam_space(req.id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Exam space deleted: id={}", req.id);

        Ok(Response::new(Empty {}))
    }

    pub async fn list_exam_spaces_by_user(
        &self,
        request: Request<ListExamSpacesRequest>,
    ) -> Result<Response<ListExamSpacesResponse>, Status> {
        let req = request.into_inner();

        let rows = self
            .exam_space_db
            .find_exam_spaces_by_user(req.user_id)
            .await
            .map_err(|e| Status::internal(e))?;

        let spaces = rows
            .into_iter()
            .map(|r| ExamSpace {
                id: r.id,
                token: r.token,
                name: r.name,
                description: r.description,
                last_updated_at: r.last_updated_at,
                member_count: r.member_count,
                paper_count: r.paper_count,
            })
            .collect();

        Ok(Response::new(ListExamSpacesResponse { spaces }))
    }

    pub async fn add_exam_space_member(
        &self,
        request: Request<AddExamSpaceMemberRequest>,
    ) -> Result<Response<AddExamSpaceMemberResponse>, Status> {
        let req = request.into_inner();

        let id = self
            .member_db
            .add_member(req.user_id, req.exam_space_id, &req.role)
            .await
            .map_err(|e| Status::internal(e))?;

        info!(
            "Member added: id={}, user_id={}, exam_space_id={}, role={}",
            id, req.user_id, req.exam_space_id, req.role
        );

        Ok(Response::new(AddExamSpaceMemberResponse { id }))
    }

    pub async fn update_exam_space_member(
        &self,
        request: Request<UpdateExamSpaceMemberRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.member_db
            .update_member_role(req.id, &req.role)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Member role updated: id={}, role={}", req.id, req.role);

        Ok(Response::new(Empty {}))
    }

    pub async fn remove_exam_space_member(
        &self,
        request: Request<RemoveExamSpaceMemberRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.member_db
            .remove_member(req.id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Member removed: id={}", req.id);

        Ok(Response::new(Empty {}))
    }

    pub async fn list_exam_space_members(
        &self,
        request: Request<ListExamSpaceMembersRequest>,
    ) -> Result<Response<ListExamSpaceMembersResponse>, Status> {
        let req = request.into_inner();

        let rows = self
            .member_db
            .find_members_by_space(req.exam_space_id)
            .await
            .map_err(|e| Status::internal(e))?;

        let members = rows
            .into_iter()
            .map(|r| ExamSpaceMember {
                id: r.id,
                user_id: r.user_id,
                exam_space_id: r.exam_space_id,
                role: r.role,
                last_accessed_at: r.last_accessed_at.unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(ListExamSpaceMembersResponse { members }))
    }
}