use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tracing::info;

use crate::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("WebSocket connection request, token: {token}");

    let Some((conn_id, rx)) = state.doc_sync.try_join(&token).await else {
        info!("on_conn_before hook rejected token: {token}");
        return Err(StatusCode::FORBIDDEN);
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, token, state, conn_id, rx)))
}

async fn handle_socket(
    socket: WebSocket,
    token: String,
    state: Arc<AppState>,
    conn_id: u64,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if sender
                    .send(Message::Text(msg.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    Message::Text(text) => {
                        info!("Received text message: {text}");
                        state.doc_sync.broadcast(&token, &text).await;
                    }
                    Message::Close(_) => {
                        info!("Client disconnected, token: {token}");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    state.doc_sync.leave(&token, conn_id).await;
}