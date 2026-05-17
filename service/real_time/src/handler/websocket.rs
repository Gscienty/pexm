use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use auth_lib::claim::Claim;
use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    response::IntoResponse,
};

use crate::{
    biz::{handler::handle_sync_stream, principal::Principal, room::cmd::DocRoomCommand},
    AppState,
};

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub async fn websocket_handler(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let (doc_id, user_id, user_name) = (1, 1, "mock_user".to_string());

    let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let claim = Claim::new(user_id, user_name, connection_id);
    let principal = Principal::new(doc_id, token, claim, connection_id);

    ws.on_upgrade(move |ws| async move {
        let docroom = if let Some(cmd) = state.room_set.get_docroom(&principal).await {
            cmd
        } else {
            let Ok(cmd): Result<DocRoomCommand, ()> =
                state.room_set.build_docroom(&principal, (), &[]).await
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
