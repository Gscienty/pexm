mod biz;
mod handler;
mod rpcclient;

use axum::{extract::FromRequestParts, routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use auth_lib::jot::Jot;

use crate::biz::room::doc_room::DocRoomCallbackMessage;
use crate::biz::room_set::RoomSet;
use crate::rpcclient::{DalClient, StorePaperContentRequest};

pub struct AppState {
    pub room_set: RoomSet<()>,
    pub jot: Arc<Jot>,
    pub dal: DalClient,
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

    let dal_addr = std::env::var("DAL_ADDR").unwrap_or_else(|_| "http://127.0.0.1:4000".to_string());
    let dal = DalClient::connect(dal_addr)
        .await
        .expect("Failed to connect to dal service");

    let state = Arc::new(AppState {
        room_set: RoomSet::new(),
        jot,
        dal,
    });

    let callback_state = state.clone();
    tokio::spawn(async move {
        loop {
            if let Some(msg) = callback_state.room_set.recv_callback().await {
                match msg {
                    DocRoomCallbackMessage::Store(id, content) => {
                        let paper_id = id as i64;
                        let mut client = callback_state.dal.clone();
                        match client
                            .store_paper_content(StorePaperContentRequest {
                                id: paper_id,
                                content,
                            })
                            .await
                        {
                            Ok(_) => info!("DocRoom {} stored to dal", paper_id),
                            Err(e) => tracing::error!("Failed to store DocRoom {}: {}", paper_id, e),
                        }
                    }
                    DocRoomCallbackMessage::PreDestory(id) => {
                        let paper_id = id as i64;

                        // 1. Remove the closed room from the active map.
                        //    Only removes if the room is still closed (not
                        //    replaced by a new WebSocket connection).
                        callback_state.room_set.remove_closed_room(id).await;
                        info!("DocRoom {} removed from active map", paper_id);

                        // 2. Persist pending content to dal.
                        //    Use get_pending_content (read-only, sync) so that
                        //    concurrent get_content calls can still find the
                        //    data while the DAL write is in flight.
                        let content = callback_state.room_set.get_pending_content(id);
                        if let Some(content) = content {
                            if !content.is_empty() {
                                let mut client = callback_state.dal.clone();
                                match client
                                    .store_paper_content(StorePaperContentRequest {
                                        id: paper_id,
                                        content,
                                    })
                                    .await
                                {
                                    Ok(_) => info!("DocRoom {} stored to dal on destory", paper_id),
                                    Err(e) => tracing::error!("Failed to store DocRoom {} on destory: {}", paper_id, e),
                                }
                            }

                            // 3. Clear pending AFTER dal confirms the write.
                            callback_state.room_set.clear_pending(id);
                        }

                        info!("DocRoom {} fully destoryed", paper_id);
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
        .route(
            "/api/doc/{id}/content",
            get(handler::content::get_paper_content),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    info!("Server listening on 0.0.0.0:3001");

    axum::serve(listener, app).await.unwrap();
}
