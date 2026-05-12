mod handler;
mod biz;

use axum::{routing::get, Router};
use biz::room::DocSyncRoomManager;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

pub struct AppState {
    pub doc_sync: Arc<DocSyncRoomManager>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "real_time=info".into()),
        )
        .init();

    let mut doc_sync = DocSyncRoomManager::new();

    let state = Arc::new(AppState {
        doc_sync: Arc::new(doc_sync),
    });

    let app = Router::new()
        .route(
            "/api/websocket/doc/{token}",
            get(handler::websocket::websocket_handler),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server listening on 0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}