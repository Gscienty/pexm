use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::sync::{
    mpsc::UnboundedSender,
    oneshot,
};

use crate::biz::principal::Principal;

pub(super) enum DocCommandMessage {
    Join(Arc<Principal>, SplitSink<WebSocket, Message>),
    Sync(Arc<Principal>, Vec<u8>),
    Leave(Arc<Principal>),
    GetContent(oneshot::Sender<Vec<u8>>),
}

/// Synchronous content cache shared between DocRoom and DocRoomCommand.
/// The DocRoom writes its final content here before exiting the run loop,
/// so that `get_content` can always return data even when the async
/// channel is no longer being polled.
type ExitContent = std::sync::Mutex<Option<Vec<u8>>>;

#[derive(Clone)]
pub struct DocRoomCommand {
    cmd: UnboundedSender<DocCommandMessage>,
    exit_content: Arc<ExitContent>,
}

impl DocRoomCommand {
    pub(super) fn new(cmd: UnboundedSender<DocCommandMessage>) -> Self {
        Self {
            cmd,
            exit_content: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Returns true if the DocRoom task has exited (receiver dropped).
    pub(crate) fn is_closed(&self) -> bool {
        self.cmd.is_closed()
    }

    /// Returns true if the room has set its exit content (exiting or exited).
    pub(crate) fn has_exit_content(&self) -> bool {
        self.exit_content.lock().unwrap().is_some()
    }

    /// Set the exit content cache. Called by DocRoom before it stops
    /// processing messages. Synchronous — no async gap.
    pub(crate) fn set_exit_content(&self, content: Vec<u8>) {
        *self.exit_content.lock().unwrap() = Some(content);
    }

    pub(crate) fn join(
        &self,
        principal: Arc<Principal>,
        outgoing: SplitSink<WebSocket, Message>,
    ) -> Result<(), ()> {
        match self.cmd.send(DocCommandMessage::Join(principal, outgoing)) {
            Ok(_) => Ok(()),
            Err(_err) => {
                // TODO
                Err(())
            }
        }
    }

    pub(crate) fn sync(&self, principal: Arc<Principal>, msg: Vec<u8>) -> Result<(), ()> {
        match self.cmd.send(DocCommandMessage::Sync(principal, msg)) {
            Ok(_) => Ok(()),
            Err(_err) => {
                // TODO
                Err(())
            }
        }
    }

    pub(crate) fn leave(&self, principal: Arc<Principal>) -> Result<(), ()> {
        match self.cmd.send(DocCommandMessage::Leave(principal)) {
            Ok(_) => Ok(()),
            Err(_err) => {
                // TODO
                Err(())
            }
        }
    }

    /// Request the current document content from the room.
    /// Checks exit_content FIRST (synchronous, no blocking). Only falls
    /// back to the async channel if the room is still active.
    pub(crate) async fn get_content(&self) -> Option<Vec<u8>> {
        // 1. Check exit content first. If set, the room has exited or is
        //    exiting — return the cached content immediately without
        //    touching the channel (which would block forever because
        //    nobody is reading from it anymore).
        if let Some(content) = self.exit_content.lock().unwrap().clone() {
            return Some(content);
        }

        // 2. Room is still active — use channel to get real-time content.
        let (tx, rx) = oneshot::channel();
        match self.cmd.send(DocCommandMessage::GetContent(tx)) {
            Ok(_) => rx.await.ok(),
            Err(_) => None,
        }
    }
}
