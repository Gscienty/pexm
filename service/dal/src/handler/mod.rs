pub mod auth;

use tonic::{Request, Response, Status};

use crate::dal::{
    dal_service_server::DalService,
    LoginByEmailRequest, LoginByEmailResponse, RegisterRequest, RegisterResponse,
};
use auth::AuthHandler;

#[derive(Clone)]
pub struct DalServiceHandler {
    auth: AuthHandler,
}

impl DalServiceHandler {
    pub fn new(auth: AuthHandler) -> Self {
        Self { auth }
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
}