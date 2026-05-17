mod biz;
mod handler;

use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::biz::room_set::RoomSet;

pub struct AppState {
    pub room_set: RoomSet<()>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "real_time=info".into()),
        )
        .init();

    let state = Arc::new(AppState {
        room_set: RoomSet::new(),
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
