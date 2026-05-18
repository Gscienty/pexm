mod biz;
mod handler;

use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::biz::room::doc_room::DocRoomCallbackMessage;
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

    let callback_state = state.clone();
    tokio::spawn(async move {
        loop {
            if let Some(msg) = callback_state.room_set.recv_callback().await {
                match msg {
                    DocRoomCallbackMessage::Store(id, content) => {
                        // TODO: store document to persistent storage
                        info!("DocRoom {} stored, size: {}", id, content.len());
                    }
                    DocRoomCallbackMessage::Destory(id) => {
                        info!("DocRoom {} destorying", id);
                        callback_state.room_set.destory_room(id).await;
                    }
                }
            }
        }
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
