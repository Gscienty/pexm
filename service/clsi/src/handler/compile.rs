use core::convert::Into;
use std::path::PathBuf;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::{
    clsi::{CompilePdfRequest, CompilePdfResponse},
    handler::default_lua_script::DEFAULT_LUA_SCRIPT,
    latex_render::{self, block::Block},
    lua_engine,
};

#[derive(Clone)]
pub struct CompileHandler {
    work_dir: PathBuf,
}

impl CompileHandler {
    pub fn new() -> Self {
        let work_dir = std::env::var("CLSI_WORK_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".clsi")
            });
        Self { work_dir }
    }
}

fn empty_doc_block() -> Block {
    Block {
        content_type: "doc".into(),
        ..Default::default()
    }
}

impl CompileHandler {
    pub async fn compile_pdf(
        &self,
        request: Request<CompilePdfRequest>,
    ) -> Result<Response<CompilePdfResponse>, Status> {
        let req = request.into_inner();
        let yjs_content = req.yjs_content;
        let lua_script = req.lua_script;
        let config = req.config;
        let title = req.title;

        let doc = match latex_render::convert(&yjs_content) {
            Ok(doc) => doc,
            Err(err) => {
                let err_msg: String = err.into();
                info!(
                    "Failed to convert yjs content (using empty doc): {}",
                    err_msg
                );
                empty_doc_block()
            }
        };

        tracing::info!("{doc:#?}");

        let script = if lua_script.is_empty() {
            DEFAULT_LUA_SCRIPT
        } else {
            &lua_script
        };

        let latex_content = match lua_engine::render_with_lua(script, &doc, &config, &title) {
            Ok(content) => content,
            Err(err) => {
                error!("Lua render error: {}", err);
                return Err(Status::invalid_argument(format!(
                    "Lua render error: {}",
                    err
                )));
            }
        };

        let job_id = uuid::Uuid::new_v4();
        let job_dir = self.work_dir.join(job_id.to_string());
        std::fs::create_dir_all(&job_dir).map_err(|e| {
            error!("Failed to create job dir {:?}: {}", job_dir, e);
            Status::internal(format!("Failed to create job dir: {}", e))
        })?;

        let job_dir_real = job_dir.canonicalize().map_err(|e| {
            error!("Failed to canonicalize {:?}: {}", job_dir, e);
            Status::internal(format!("Failed to resolve path: {}", e))
        })?;

        let tex_path = job_dir_real.join("input.tex");
        tokio::fs::write(&tex_path, latex_content)
            .await
            .map_err(|e| {
                error!("Failed to write tex file: {}", e);
                Status::internal(format!("Failed to write tex file: {}", e))
            })?;

        info!("Wrote tex file to {:?}", tex_path);

        let job_dir_str = job_dir_real.to_string_lossy().to_string();
        let cmd = format!(
            "podman run --rm --entrypoint sh -v {}:/workspace pexm/latex -c 'cd /workspace && xelatex -interaction=nonstopmode input.tex'",
            job_dir_str,
        );

        info!("Running: {}", cmd);

        let output = tokio::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await
            .map_err(|e| {
                error!("Failed to execute podman: {}", e);
                Status::internal(format!("Failed to execute podman: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("xelatex failed:\nstdout: {}\nstderr: {}", stdout, stderr);

            if let Err(e) = tokio::fs::remove_dir_all(&job_dir_real).await {
                error!("Failed to clean up job dir {:?}: {}", job_dir_real, e);
            }

            return Ok(Response::new(CompilePdfResponse {
                pdf_data: vec![],
                success: false,
                error: format!(
                    "xelatex compilation failed:\nstdout: {}\nstderr: {}",
                    stdout, stderr
                ),
            }));
        }

        let pdf_path = job_dir_real.join("input.pdf");
        let pdf_data = tokio::fs::read(&pdf_path).await.map_err(|e| {
            error!("Failed to read pdf file {:?}: {}", pdf_path, e);
            Status::internal(format!("Failed to read pdf file: {}", e))
        })?;

        info!("Generated PDF, {} bytes", pdf_data.len());

        if let Err(e) = tokio::fs::remove_dir_all(&job_dir_real).await {
            error!("Failed to clean up job dir {:?}: {}", job_dir_real, e);
        }

        Ok(Response::new(CompilePdfResponse {
            pdf_data,
            success: true,
            error: String::new(),
        }))
    }
}
