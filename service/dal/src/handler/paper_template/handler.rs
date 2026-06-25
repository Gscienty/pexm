use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use crate::dal::{
    CreatePaperTemplateRequest, CreatePaperTemplateResponse, DeletePaperTemplateRequest,
    Empty, GetPaperTemplateRequest, GetPaperTemplateResponse,
    ListPaperTemplatesRequest, ListPaperTemplatesResponse,
    LoadPaperTemplateContentRequest, LoadPaperTemplateContentResponse,
    PaperTemplate, StorePaperTemplateContentRequest, StorePaperTemplateContentResponse,
    UpdatePaperTemplateRequest,
};
use crate::db::PaperTemplateRepo;

#[derive(Clone)]
pub struct PaperTemplateHandler {
    template_db: Arc<dyn PaperTemplateRepo>,
}

impl PaperTemplateHandler {
    pub fn new(template_db: Arc<dyn PaperTemplateRepo>) -> Self {
        Self { template_db }
    }

    pub async fn create_paper_template(
        &self,
        request: Request<CreatePaperTemplateRequest>,
    ) -> Result<Response<CreatePaperTemplateResponse>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let description = req.description.trim().to_string();

        let id = self
            .template_db
            .create_paper_template(&name, &description)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper template created: id={}, name={}", id, name);
        Ok(Response::new(CreatePaperTemplateResponse { id }))
    }

    pub async fn update_paper_template(
        &self,
        request: Request<UpdatePaperTemplateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let description = req.description.trim().to_string();
        let preview = if req.preview.is_empty() { None } else { Some(req.preview.as_slice()) };

        self.template_db
            .update_paper_template(req.id, &name, &description, preview)
            .await
            .map_err(|e| Status::internal(e))?;

        info!(
            "Paper template updated: id={}, preview_size={}",
            req.id,
            req.preview.len()
        );
        Ok(Response::new(Empty {}))
    }

    pub async fn delete_paper_template(
        &self,
        request: Request<DeletePaperTemplateRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.template_db
            .delete_paper_template(req.id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper template deleted: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    pub async fn get_paper_template(
        &self,
        request: Request<GetPaperTemplateRequest>,
    ) -> Result<Response<GetPaperTemplateResponse>, Status> {
        let req = request.into_inner();

        let t = self
            .template_db
            .find_paper_template_by_id(req.id)
            .await
            .map_err(|e| Status::internal(e))?
            .ok_or_else(|| Status::not_found("paper template not found"))?;

        Ok(Response::new(GetPaperTemplateResponse {
            template: Some(PaperTemplate {
                id: t.id,
                name: t.name,
                description: t.description,
                created_at: t.created_at,
                updated_at: t.updated_at,
                preview: t.preview.unwrap_or_default(),
            }),
        }))
    }

    pub async fn list_paper_templates(
        &self,
        _request: Request<ListPaperTemplatesRequest>,
    ) -> Result<Response<ListPaperTemplatesResponse>, Status> {
        let rows = self
            .template_db
            .find_all_paper_templates()
            .await
            .map_err(|e| Status::internal(e))?;

        let templates = rows
            .into_iter()
            .map(|t| PaperTemplate {
                id: t.id,
                name: t.name,
                description: t.description,
                created_at: t.created_at,
                updated_at: t.updated_at,
                preview: t.preview.unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(ListPaperTemplatesResponse { templates }))
    }

    pub async fn store_paper_template_content(
        &self,
        request: Request<StorePaperTemplateContentRequest>,
    ) -> Result<Response<StorePaperTemplateContentResponse>, Status> {
        let req = request.into_inner();

        self.template_db
            .store_paper_template_content(req.id, &req.content)
            .await
            .map_err(|e| Status::internal(e))?;

        info!(
            "Paper template content stored: id={}, size={}",
            req.id,
            req.content.len()
        );
        Ok(Response::new(StorePaperTemplateContentResponse {}))
    }

    pub async fn load_paper_template_content(
        &self,
        request: Request<LoadPaperTemplateContentRequest>,
    ) -> Result<Response<LoadPaperTemplateContentResponse>, Status> {
        let req = request.into_inner();

        let content = self
            .template_db
            .load_paper_template_content(req.id)
            .await
            .map_err(|e| Status::internal(e))?
            .unwrap_or_default();

        Ok(Response::new(LoadPaperTemplateContentResponse { content }))
    }
}
