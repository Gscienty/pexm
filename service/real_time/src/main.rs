mod biz;
mod handler;

use axum::{extract::FromRequestParts, routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use auth_lib::jot::Jot;

use crate::biz::room::doc_room::DocRoomCallbackMessage;
use crate::biz::room_set::RoomSet;

pub struct AppState {
    pub room_set: RoomSet<()>,
    pub jot: Arc<Jot>,
}

/// Local newtype to satisfy orphan rules for FromRequestParts.
pub struct AppJotProvider(auth_lib::JotProvider);

impl FromRequestParts<Arc<AppState>> for AppJotProvider {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        Ok(AppJotProvider(auth_lib::JotProvider(state.jot.clone())))
    }
}

impl std::ops::Deref for AppJotProvider {
    type Target = Jot;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "real_time=info".into()),
        )
        .init();

    let pkey = std::fs::read("../../keys/ed25519_private.pem").expect("Failed to read private key");
    let pubkey = std::fs::read("../../keys/ed25519_public.pem").expect("Failed to read public key");
    let jot = Arc::new(Jot::new(&pkey, &pubkey).expect("Failed to create Jot from keys"));

    let state = Arc::new(AppState {
        room_set: RoomSet::new(),
        jot,
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    info!("Server listening on 0.0.0.0:3001");

    axum::serve(listener, app).await.unwrap();
}
