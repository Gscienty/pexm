pub mod compile;
mod default_lua_script;

use tonic::{Request, Response, Status};

use crate::clsi::{CompilePdfRequest, CompilePdfResponse, clsi_service_server::ClsiService};
use compile::CompileHandler;

#[derive(Clone)]
pub struct ClsiServiceHandler {
    compile: CompileHandler,
}

impl ClsiServiceHandler {
    pub fn new() -> Self {
        Self {
            compile: CompileHandler::new(),
        }
    }
}

#[tonic::async_trait]
impl ClsiService for ClsiServiceHandler {
    async fn compile_pdf(
        &self,
        request: Request<CompilePdfRequest>,
    ) -> Result<Response<CompilePdfResponse>, Status> {
        self.compile.compile_pdf(request).await
    }
}
