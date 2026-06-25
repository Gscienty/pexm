use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::info;

use crate::dal::{
    CreateDirectoryRequest, CreateDirectoryResponse, CreatePaperRequest, CreatePaperResponse,
    DeleteDirectoryRequest, DeletePaperRequest, Directory, DirectoryBreadcrumbItem,
    Empty, GetDirectoryBreadcrumbRequest, GetDirectoryBreadcrumbResponse,
    GetPaperBreadcrumbRequest, GetPaperBreadcrumbResponse,
    ListDirectoryChildrenRequest, ListDirectoryChildrenResponse,
    MoveDirectoryRequest, MovePaperRequest, Paper,
    SetPaperTemplateRequest, SetPaperTemplateResponse,
    StorePaperContentRequest, StorePaperContentResponse,
    LoadPaperContentRequest, LoadPaperContentResponse,
    UpdateDirectoryRequest, UpdatePaperRequest,
};
use crate::db::{DirectoryRepo, PaperRepo};

#[derive(Clone)]
pub struct DirectoryHandler {
    directory_db: Arc<dyn DirectoryRepo>,
    paper_db: Arc<dyn PaperRepo>,
}

impl DirectoryHandler {
    pub fn new(directory_db: Arc<dyn DirectoryRepo>, paper_db: Arc<dyn PaperRepo>) -> Self {
        Self {
            directory_db,
            paper_db,
        }
    }

    pub async fn create_directory(
        &self,
        request: Request<CreateDirectoryRequest>,
    ) -> Result<Response<CreateDirectoryResponse>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }
        let parent_id = if req.parent_id == 0 { None } else { Some(req.parent_id) };

        let id = self
            .directory_db
            .create_directory(req.exam_space_id, parent_id, &name)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Directory created: id={}, name={}, space={}", id, name, req.exam_space_id);
        Ok(Response::new(CreateDirectoryResponse { id }))
    }

    pub async fn update_directory(
        &self,
        request: Request<UpdateDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }

        self.directory_db
            .update_directory(req.id, &name)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Directory updated: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    pub async fn delete_directory(
        &self,
        request: Request<DeleteDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.directory_db
            .delete_directory(req.id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Directory deleted: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    pub async fn move_directory(
        &self,
        request: Request<MoveDirectoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let new_parent_id = if req.new_parent_id == 0 { None } else { Some(req.new_parent_id) };

        self.directory_db
            .move_directory(req.id, new_parent_id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Directory moved: id={}, new_parent_id={:?}", req.id, new_parent_id);
        Ok(Response::new(Empty {}))
    }

    pub async fn list_directory_children(
        &self,
        request: Request<ListDirectoryChildrenRequest>,
    ) -> Result<Response<ListDirectoryChildrenResponse>, Status> {
        let req = request.into_inner();
        let parent_id = if req.parent_id == 0 { None } else { Some(req.parent_id) };

        let dirs = self
            .directory_db
            .find_subdirectories(req.exam_space_id, parent_id)
            .await
            .map_err(|e| Status::internal(e))?;

        let papers = self
            .paper_db
            .find_papers_by_directory(req.exam_space_id, parent_id)
            .await
            .map_err(|e| Status::internal(e))?;

        let directories = dirs
            .into_iter()
            .map(|d| Directory {
                id: d.id,
                exam_space_id: d.exam_space_id,
                parent_id: d.parent_id.unwrap_or(0),
                name: d.name,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect();

        let papers = papers
            .into_iter()
            .map(|p| Paper {
                id: p.id,
                exam_space_id: p.exam_space_id,
                directory_id: p.directory_id.unwrap_or(0),
                title: p.title,
                created_at: p.created_at,
                updated_at: p.updated_at,
                template_id: p.template_id.unwrap_or(0),
                template_config: p.template_config.unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(ListDirectoryChildrenResponse {
            directories,
            papers,
        }))
    }

    pub async fn get_directory_breadcrumb(
        &self,
        request: Request<GetDirectoryBreadcrumbRequest>,
    ) -> Result<Response<GetDirectoryBreadcrumbResponse>, Status> {
        let req = request.into_inner();

        let items = self
            .directory_db
            .get_breadcrumb(req.directory_id)
            .await
            .map_err(|e| Status::internal(e))?;

        let items = items
            .into_iter()
            .map(|i| DirectoryBreadcrumbItem {
                id: i.id,
                name: i.name,
            })
            .collect();

        Ok(Response::new(GetDirectoryBreadcrumbResponse { items }))
    }

    pub async fn create_paper(
        &self,
        request: Request<CreatePaperRequest>,
    ) -> Result<Response<CreatePaperResponse>, Status> {
        let req = request.into_inner();
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(Status::invalid_argument("title is required"));
        }
        let directory_id = if req.directory_id == 0 { None } else { Some(req.directory_id) };

        let id = self
            .paper_db
            .create_paper(req.exam_space_id, directory_id, &title)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper created: id={}, title={}, space={}", id, title, req.exam_space_id);
        Ok(Response::new(CreatePaperResponse { id }))
    }

    pub async fn update_paper(
        &self,
        request: Request<UpdatePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(Status::invalid_argument("title is required"));
        }

        self.paper_db
            .update_paper(req.id, &title)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper updated: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    pub async fn delete_paper(
        &self,
        request: Request<DeletePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();

        self.paper_db
            .delete_paper(req.id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper deleted: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    pub async fn move_paper(
        &self,
        request: Request<MovePaperRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let new_directory_id = if req.new_directory_id == 0 { None } else { Some(req.new_directory_id) };

        self.paper_db
            .move_paper(req.id, new_directory_id)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper moved: id={}, new_directory_id={:?}", req.id, new_directory_id);
        Ok(Response::new(Empty {}))
    }

    pub async fn get_paper_breadcrumb(
        &self,
        request: Request<GetPaperBreadcrumbRequest>,
    ) -> Result<Response<GetPaperBreadcrumbResponse>, Status> {
        let req = request.into_inner();

        let paper = self
            .paper_db
            .find_paper_by_id(req.paper_id)
            .await
            .map_err(|e| Status::internal(e))?
            .ok_or_else(|| Status::not_found("paper not found"))?;

        let directory_items = if let Some(dir_id) = paper.directory_id {
            self.directory_db
                .get_breadcrumb(dir_id)
                .await
                .map_err(|e| Status::internal(e))?
                .into_iter()
                .map(|i| DirectoryBreadcrumbItem {
                    id: i.id,
                    name: i.name,
                })
                .collect()
        } else {
            vec![]
        };

        Ok(Response::new(GetPaperBreadcrumbResponse {
            directory_items,
            paper: Some(Paper {
                id: paper.id,
                exam_space_id: paper.exam_space_id,
                directory_id: paper.directory_id.unwrap_or(0),
                title: paper.title,
                created_at: paper.created_at,
                updated_at: paper.updated_at,
                template_id: paper.template_id.unwrap_or(0),
                template_config: paper.template_config.unwrap_or_default(),
            }),
        }))
    }

    pub async fn store_paper_content(
        &self,
        request: Request<StorePaperContentRequest>,
    ) -> Result<Response<StorePaperContentResponse>, Status> {
        let req = request.into_inner();

        self.paper_db
            .store_paper_content(req.id, &req.content)
            .await
            .map_err(|e| Status::internal(e))?;

        info!("Paper content stored: id={}, size={}", req.id, req.content.len());
        Ok(Response::new(StorePaperContentResponse {}))
    }

    pub async fn load_paper_content(
        &self,
        request: Request<LoadPaperContentRequest>,
    ) -> Result<Response<LoadPaperContentResponse>, Status> {
        let req = request.into_inner();

        let content = self
            .paper_db
            .load_paper_content(req.id)
            .await
            .map_err(|e| Status::internal(e))?
            .unwrap_or_default();

        Ok(Response::new(LoadPaperContentResponse { content }))
    }

    pub async fn set_paper_template(
        &self,
        request: Request<SetPaperTemplateRequest>,
    ) -> Result<Response<SetPaperTemplateResponse>, Status> {
        let req = request.into_inner();
        let template_id = if req.template_id == 0 { None } else { Some(req.template_id) };
        let template_config = if req.template_config.is_empty() { None } else { Some(req.template_config.as_slice()) };

        self.paper_db
            .set_paper_template(req.id, template_id, template_config)
            .await
            .map_err(|e| Status::internal(e))?;

        info!(
            "Paper template set: id={}, template_id={:?}, config_size={}",
            req.id,
            template_id,
            req.template_config.len()
        );
        Ok(Response::new(SetPaperTemplateResponse {}))
    }
}
