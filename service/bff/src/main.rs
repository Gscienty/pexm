mod rpcclient;
mod config;
mod handler;

use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::config::Config;
use crate::rpcclient::DalClient;

pub struct AppState {
    pub dal: DalClient,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bff=info".into()),
        )
        .init();

    let config = Config::load()?;
    info!("Loaded config: {:?}", config);

    let dal = DalClient::connect(config.dal.addr.clone()).await?;
    info!("Connected to dal service at {}", config.dal.addr);

    let state = Arc::new(AppState { dal });

    let app = Router::new()
        .merge(handler::routes(state.clone()))
        .layer(CorsLayer::permissive());

    let addr = config.server_addr();
    info!("BFF server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}