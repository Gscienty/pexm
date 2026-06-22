use tonic::{Request, Response, Status};

use crate::clsi::CompilePdfRequest;
use crate::clsi::CompilePdfResponse;

#[derive(Clone)]
pub struct CompileHandler;

impl CompileHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CompileHandler {
    pub async fn compile_pdf(
        &self,
        _request: Request<CompilePdfRequest>,
    ) -> Result<Response<CompilePdfResponse>, Status> {
        // TODO: 实现 doc json 编译为 pdf 的逻辑
        Ok(Response::new(CompilePdfResponse {
            pdf_data: vec![],
            success: false,
            error: "not implemented".to_string(),
        }))
    }
}
