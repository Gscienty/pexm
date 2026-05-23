mod db;
mod handler;

use std::sync::Arc;

use auth_lib::jot::Jot;
use db::sqlite::SqliteDb;
use db::{DbPool, ExamSpaceMemberRepo, ExamSpaceRepo};
use handler::auth::AuthHandler;
use handler::exam_space::ExamSpaceHandler;
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
    let exam_space_db: Arc<dyn ExamSpaceRepo> = sqlite.clone();
    let member_db: Arc<dyn ExamSpaceMemberRepo> = sqlite;

    let pkey = include_bytes!("../../../keys/ed25519_private.pem");
    let pubkey = include_bytes!("../../../keys/ed25519_public.pem");
    let jot = Arc::new(Jot::new(pkey, pubkey).map_err(|e| e.to_string())?);

    let auth_handler = AuthHandler::new(db, jot);
    let exam_space_handler = ExamSpaceHandler::new(exam_space_db, member_db);
    let dal_service = DalServiceHandler::new(auth_handler, exam_space_handler);

    let addr = "0.0.0.0:4000".parse()?;

    info!("Dal gRPC server listening on {}", addr);

    Server::builder()
        .add_service(DalServiceServer::new(dal_service))
        .serve(addr)
        .await?;

    Ok(())
}