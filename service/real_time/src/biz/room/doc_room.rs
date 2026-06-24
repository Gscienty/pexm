use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::interval,
};
use y_octo::Doc;

use crate::biz::{doc::document::Document, principal::Principal};

use super::{cmd::DocCommandMessage, cmd::DocRoomCommand};

pub enum DocRoomCallbackMessage {
    /// Periodic store: persist content to dal.
    Store(u64, Vec<u8>),
    /// Room exited (all connections gone). Content has already been stashed
    /// in pending_content by the DocRoom before sending this callback.
    /// The receiver should: remove closed room from active → persist pending
    /// to dal → clear pending.
    PreDestory(u64),
}

/// Synchronous pending content map. Uses std::sync::Mutex instead of
/// tokio::sync::RwLock so that the DocRoom can write to it synchronously
/// before dropping the channel receiver — no async gap.
type PendingMap = Arc<std::sync::Mutex<HashMap<u64, Vec<u8>>>>;

pub struct DocRoom {
    id: u64,
    callback: UnboundedSender<DocRoomCallbackMessage>,
    pending: PendingMap,
    cmd: DocRoomCommand,
    doc: Document,
    recv: Option<UnboundedReceiver<DocCommandMessage>>,
}

impl DocRoom {
    fn new(
        callback: UnboundedSender<DocRoomCallbackMessage>,
        id: u64,
        name: String,
        doc: Doc,
        pending: PendingMap,
        cmd: DocRoomCommand,
        recv: UnboundedReceiver<DocCommandMessage>,
    ) -> Self {
        let doc = Document::new(name, doc);
        Self {
            id,
            callback,
            pending,
            cmd,
            doc,
            recv: Some(recv),
        }
    }

    pub fn create(
        principal: &Principal,
        doc: Doc,
        callback: UnboundedSender<DocRoomCallbackMessage>,
        pending: PendingMap,
    ) -> DocRoomCommand {
        let id = principal.get_id();
        let name = principal.get_doc_name().to_owned();

        let (outgoing, incoming) = unbounded_channel();
        let cmd = DocRoomCommand::new(outgoing);

        let cmd_clone = cmd.clone();
        tokio::spawn(async move {
            let mut docroom = Self::new(callback, id, name, doc, pending, cmd_clone, incoming);

            docroom.run().await;
        });

        cmd
    }

    async fn run(&mut self) {
        let mut timer = interval(Duration::from_secs(10));
        // Take the receiver out of Option so we can use it in select!,
        // and later drop it to close the channel.
        let mut recv = self.recv.take().unwrap();

        loop {
            tokio::select! {
                msg = recv.recv() => {
                    if let Some(msg) = msg {
                        self.handle_message(msg).await;
                    } else {
                        break;
                    }
                },
                _  = timer.tick() => {
                    self.callback_store();
                }
            };

            if self.doc.is_connection_empty() {
                break;
            }
        }

        // Room exited. Encode the final content once.
        let content = self.doc.encode_state().unwrap_or_default();

        // 1. Synchronously set the exit-content cache on DocRoomCommand.
        //    After this, cmd.get_content() returns immediately from the
        //    cache instead of trying the (now-dead) channel.
        self.cmd.set_exit_content(content.clone());

        // 2. Synchronously stash content to pending. Uses std::sync::Mutex
        //    so there is no async gap — the content is available immediately
        //    for concurrent get_content / take_pending_content calls.
        {
            let mut pending = self.pending.lock().unwrap();
            pending.insert(self.id, content);
        }

        // 3. Drop the channel receiver. This closes the channel so that:
        //    - cmd.is_closed() returns true
        //    - cmd.send() returns Err (no more messages accepted)
        //    - Any oneshot senders in pending channel messages are dropped,
        //      unblocking any rx.await that was waiting for a reply.
        //      (We drain and drop_reply first to be explicit.)
        drop(recv);

        // 4. Notify the callback handler so it can remove us from the
        //    active map and persist the pending content to dal.
        if let Err(err) = self
            .callback
            .send(DocRoomCallbackMessage::PreDestory(self.id))
        {
            tracing::error!("callback pre-destory failed: {}", err);
        }

        self.doc.destory();
    }

    fn callback_store(&self) {
        if !self.doc.should_store() {
            return;
        }

        if let Ok(content) = self.doc.encode_state() {
            if let Err(err) = self
                .callback
                .send(DocRoomCallbackMessage::Store(self.id, content))
            {
                tracing::error!("callback store failed: {}", err);
                return;
            }

            self.doc.stored_notice();
        };
    }

    async fn handle_message(&mut self, msg: DocCommandMessage) {
        match msg {
            DocCommandMessage::Join(principal, connection) => {
                if let Err(_err) = self.doc.connect(&principal, connection).await {
                    // TODO log error
                    self.doc.disconnect(principal.get_connection_id());
                }
            }

            DocCommandMessage::Leave(principal) => {
                self.doc.disconnect(principal.get_connection_id());
            }

            DocCommandMessage::Sync(principal, message) => {
                if let Err(_err) = self.doc.handle_message(&principal, &message).await {
                    // TODO log error
                    self.doc.disconnect(principal.get_connection_id());
                }
            }

            DocCommandMessage::GetContent(reply) => {
                let content = self.doc.encode_state().unwrap_or_default();
                let _ = reply.send(content);
            }
        }
    }
}
