mod db;
mod handler;

use std::sync::Arc;

use auth_lib::jot::Jot;
use db::sqlite::SqliteDb;
use db::{DbPool, ExamSpaceMemberRepo, ExamSpaceRepo, DirectoryRepo, PaperRepo, PaperTemplateRepo};
use handler::auth::AuthHandler;
use handler::exam_space::ExamSpaceHandler;
use handler::directory::DirectoryHandler;
use handler::paper_template::PaperTemplateHandler;
use handler::DalServiceHandler;
use tonic::transport::Server;
use tracing::info;

pub mod dal {
    tonic::include_proto!("dal");
}

use dal::dal_service_server::DalServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dal=info".into()),
        )
        .init();

    let sqlite = Arc::new(SqliteDb::new("dal.db")?);

    let db: DbPool = sqlite.clone() as Arc<dyn db::UserRepo>;
    let db_for_auth = db.clone();
    let exam_space_db: Arc<dyn ExamSpaceRepo> = sqlite.clone();
    let member_db: Arc<dyn ExamSpaceMemberRepo> = sqlite.clone();
    let directory_db: Arc<dyn DirectoryRepo> = sqlite.clone();
    let paper_db: Arc<dyn PaperRepo> = sqlite.clone();
    let paper_template_db: Arc<dyn PaperTemplateRepo> = sqlite;

    let pkey = include_bytes!("../../../keys/ed25519_private.pem");
    let pubkey = include_bytes!("../../../keys/ed25519_public.pem");
    let jot = Arc::new(Jot::new(pkey, pubkey).map_err(|e| e.to_string())?);

    let auth_handler = AuthHandler::new(db_for_auth, jot);
    let exam_space_handler = ExamSpaceHandler::new(exam_space_db, member_db, db);
    let directory_handler = DirectoryHandler::new(directory_db, paper_db);
    let paper_template_handler = PaperTemplateHandler::new(paper_template_db);
    let dal_service = DalServiceHandler::new(auth_handler, exam_space_handler, directory_handler, paper_template_handler);

    let addr = "0.0.0.0:4000".parse()?;

    info!("Dal gRPC server listening on {}", addr);

    Server::builder()
        .add_service(DalServiceServer::new(dal_service))
        .serve(addr)
        .await?;

    Ok(())
}