use std::sync::Arc;

use axum::extract::ws::WebSocket;
use futures::stream::StreamExt;

use crate::biz::{principal::Principal, room::cmd::DocRoomCommand};

pub async fn handle_sync_stream(
    docroom_cmd: DocRoomCommand,
    principal: Principal,
    ws: WebSocket,
) -> Result<(), ()> {
    let principal = Arc::new(principal);

    let (outgoing, mut incoming) = ws.split();

    if let Err(_err) = docroom_cmd.join(principal.clone(), outgoing) {
        return Err(());
    }

    while let Some(msg) = incoming.next().await {
        let Ok(msg) = msg else {
            break;
        };

        if let Err(_err) = docroom_cmd.sync(principal.clone(), msg.into_data().into()) {
            break;
        }
    }

    if let Err(_err) = docroom_cmd.leave(principal) {
        return Err(());
    }

    Ok(())
}
