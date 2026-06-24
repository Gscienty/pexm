mod handler;

use handler::ClsiServiceHandler;
use tonic::transport::Server;
use tracing::info;

pub mod clsi {
    tonic::include_proto!("clsi");
}

use clsi::clsi_service_server::ClsiServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clsi=info".into()),
        )
        .init();

    let clsi_service = ClsiServiceHandler::new();

    let addr = "0.0.0.0:5001".parse()?;

    info!("Clsi gRPC server listening on {}", addr);

    Server::builder()
        .add_service(ClsiServiceServer::new(clsi_service))
        .serve(addr)
        .await?;

    Ok(())
}
