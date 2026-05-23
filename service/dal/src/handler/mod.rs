pub mod auth;
pub mod exam_space;

use tonic::{Request, Response, Status};

use crate::dal::{
    dal_service_server::DalService,
    AddExamSpaceMemberRequest, AddExamSpaceMemberResponse, CreateExamSpaceRequest,
    CreateExamSpaceResponse, DeleteExamSpaceRequest, Empty, ListExamSpaceMembersRequest,
    ListExamSpaceMembersResponse, ListExamSpacesRequest, ListExamSpacesResponse,
    LoginByEmailRequest, LoginByEmailResponse, RegisterRequest, RegisterResponse,
    RemoveExamSpaceMemberRequest, UpdateExamSpaceMemberRequest, UpdateExamSpaceRequest,
};
use auth::AuthHandler;
use exam_space::ExamSpaceHandler;

#[derive(Clone)]
pub struct DalServiceHandler {
    auth: AuthHandler,
    exam_space: ExamSpaceHandler,
}

impl DalServiceHandler {
    pub fn new(auth: AuthHandler, exam_space: ExamSpaceHandler) -> Self {
        Self { auth, exam_space }
    }
}

#[tonic::async_trait]
impl DalService for DalServiceHandler {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        self.auth.register(request).await
    }

    async fn login_by_email(
        &self,
        request: Request<LoginByEmailRequest>,
    ) -> Result<Response<LoginByEmailResponse>, Status> {
        self.auth.login_by_email(request).await
    }

    async fn create_exam_space(
        &self,
        request: Request<CreateExamSpaceRequest>,
    ) -> Result<Response<CreateExamSpaceResponse>, Status> {
        self.exam_space.create_exam_space(request).await
    }

    async fn update_exam_space(
        &self,
        request: Request<UpdateExamSpaceRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.exam_space.update_exam_space(request).await
    }

    async fn delete_exam_space(
        &self,
        request: Request<DeleteExamSpaceRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.exam_space.delete_exam_space(request).await
    }

    async fn list_exam_spaces_by_user(
        &self,
        request: Request<ListExamSpacesRequest>,
    ) -> Result<Response<ListExamSpacesResponse>, Status> {
        self.exam_space.list_exam_spaces_by_user(request).await
    }

    async fn add_exam_space_member(
        &self,
        request: Request<AddExamSpaceMemberRequest>,
    ) -> Result<Response<AddExamSpaceMemberResponse>, Status> {
        self.exam_space.add_exam_space_member(request).await
    }

    async fn update_exam_space_member(
        &self,
        request: Request<UpdateExamSpaceMemberRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.exam_space.update_exam_space_member(request).await
    }

    async fn remove_exam_space_member(
        &self,
        request: Request<RemoveExamSpaceMemberRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.exam_space.remove_exam_space_member(request).await
    }

    async fn list_exam_space_members(
        &self,
        request: Request<ListExamSpaceMembersRequest>,
    ) -> Result<Response<ListExamSpaceMembersResponse>, Status> {
        self.exam_space.list_exam_space_members(request).await
    }
}