use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::sync::mpsc::UnboundedSender;

use crate::biz::principal::Principal;

pub(super) enum DocCommandMessage {
    Join(Arc<Principal>, SplitSink<WebSocket, Message>),
    Sync(Arc<Principal>, Vec<u8>),
    Leave(Arc<Principal>),
}

#[derive(Clone)]
pub struct DocRoomCommand {
    cmd: UnboundedSender<DocCommandMessage>,
}

impl DocRoomCommand {
    pub(super) fn new(cmd: UnboundedSender<DocCommandMessage>) -> Self {
        Self { cmd }
    }

    /// Returns true if the DocRoom task has exited (receiver dropped).
    pub(crate) fn is_closed(&self) -> bool {
        self.cmd.is_closed()
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
}
