pub mod auth;
pub mod exam_space;
pub mod directory;

use tonic::{Request, Response, Status};

use crate::dal::{
    dal_service_server::DalService,
    AddExamSpaceMemberRequest, AddExamSpaceMemberResponse, CreateDirectoryRequest,
    CreateDirectoryResponse, CreateExamSpaceRequest, CreateExamSpaceResponse,
    CreatePaperRequest, CreatePaperResponse, DeleteDirectoryRequest, DeletePaperRequest,
    DeleteExamSpaceRequest, Empty,
    GetDirectoryBreadcrumbRequest, GetDirectoryBreadcrumbResponse,
    GetExamSpaceByTokenRequest, GetExamSpaceByTokenResponse,
    GetPaperBreadcrumbRequest, GetPaperBreadcrumbResponse,
    ListDirectoryChildrenRequest, ListDirectoryChildrenResponse,
    ListExamSpaceMembersRequest, ListExamSpaceMembersResponse,
    ListExamSpacesRequest, ListExamSpacesResponse,
    LoginByEmailRequest, LoginByEmailResponse, MoveDirectoryRequest, MovePaperRequest,
    RegisterRequest, RegisterResponse, RemoveExamSpaceMemberRequest,
    UpdateDirectoryRequest, UpdateExamSpaceMemberRequest, UpdateExamSpaceRequest,
    UpdatePaperRequest,
};
use auth::AuthHandler;
use exam_space::ExamSpaceHandler;
use directory::DirectoryHandler;

#[derive(Clone)]
pub struct DalServiceHandler {
    auth: AuthHandler,
    exam_space: ExamSpaceHandler,
    directory: DirectoryHandler,
}

impl DalServiceHandler {
    pub fn new(auth: AuthHandler, exam_space: ExamSpaceHandler, directory: DirectoryHandler) -> Self {
        Self { auth, exam_space, directory }
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

    async fn get_exam_space_by_token(
        &self,
        request: Request<GetExamSpaceByTokenRequest>,
    ) -> Result<Response<GetExamSpaceByTokenResponse>, Status> {
        self.exam_space.get_exam_space_by_token(request).await
    }

    // --- Directory ---

    async fn create_directory(
        &self,
        request: Request<CreateDirectoryRequest>,
    ) -> Result<Response<CreateDirectoryResponse>, Status> {
        self.directory.create_directory(request).await
    }

    async fn update_directory(
        &self,
        request: Request<UpdateDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.update_directory(request).await
    }

    async fn delete_directory(
        &self,
        request: Request<DeleteDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.delete_directory(request).await
    }

    async fn move_directory(
        &self,
        request: Request<MoveDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.move_directory(request).await
    }

    async fn list_directory_children(
        &self,
        request: Request<ListDirectoryChildrenRequest>,
    ) -> Result<Response<ListDirectoryChildrenResponse>, Status> {
        self.directory.list_directory_children(request).await
    }

    async fn get_directory_breadcrumb(
        &self,
        request: Request<GetDirectoryBreadcrumbRequest>,
    ) -> Result<Response<GetDirectoryBreadcrumbResponse>, Status> {
        self.directory.get_directory_breadcrumb(request).await
    }

    // --- Paper ---

    async fn create_paper(
        &self,
        request: Request<CreatePaperRequest>,
    ) -> Result<Response<CreatePaperResponse>, Status> {
        self.directory.create_paper(request).await
    }

    async fn update_paper(
        &self,
        request: Request<UpdatePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.update_paper(request).await
    }

    async fn delete_paper(
        &self,
        request: Request<DeletePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.delete_paper(request).await
    }

    async fn move_paper(
        &self,
        request: Request<MovePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        self.directory.move_paper(request).await
    }

    async fn get_paper_breadcrumb(
        &self,
        request: Request<GetPaperBreadcrumbRequest>,
    ) -> Result<Response<GetPaperBreadcrumbResponse>, Status> {
        self.directory.get_paper_breadcrumb(request).await
    }
}