use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use auth_lib::AuthClaim;
use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
};

use crate::{
    biz::{handler::handle_sync_stream, principal::Principal, room::cmd::DocRoomCommand},
    rpcclient::LoadPaperContentRequest,
    AppJotProvider, AppState,
};

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub async fn websocket_handler(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    AuthClaim(claim, _): AuthClaim<AppJotProvider>,
) -> impl IntoResponse {
    let paper_id: i64 = token.parse().unwrap_or(1);
    let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let principal = Principal::new(paper_id as u64, token, claim, connection_id);

    ws.on_upgrade(move |ws| async move {
        let docroom = if let Some(cmd) = state.room_set.get_docroom(&principal).await {
            cmd
        } else {
            // Check pending content first (room recently exited, not yet persisted to dal).
            let doc_binary = if let Some(content) = state.room_set.take_pending_content(paper_id as u64).await {
                tracing::info!("Using pending content for paper {}", paper_id);
                content
            } else {
                // Load from dal.
                let mut client = state.dal.clone();
                match client
                    .load_paper_content(LoadPaperContentRequest { id: paper_id })
                    .await
                {
                    Ok(resp) => {
                        let content = resp.into_inner().content;
                        if content.is_empty() {
                            vec![]
                        } else {
                            content
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load paper content for {}: {}", paper_id, e);
                        vec![]
                    }
                }
            };

            let Ok(cmd): Result<DocRoomCommand, ()> =
                state.room_set.build_docroom(&principal, (), &doc_binary).await
            else {
                tracing::error!("Failed to build docroom");
                return;
            };
            cmd
        };

        if handle_sync_stream(docroom, principal, ws).await.is_err() {
            tracing::error!("Sync stream error");
        }
    })
}
