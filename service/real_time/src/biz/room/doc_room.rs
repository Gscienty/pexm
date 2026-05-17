use std::time::Duration;

use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::interval,
};
use y_octo::Doc;

use crate::biz::{doc::document::Document, principal::Principal};

use super::{cmd::DocCommandMessage, cmd::DocRoomCommand};

pub enum DocRoomCallbackMessage {
    Store(u64, Vec<u8>),
    Destory(u64),
}

pub struct DocRoom {
    id: u64,
    callback: UnboundedSender<DocRoomCallbackMessage>,
    doc: Document,
    recv: UnboundedReceiver<DocCommandMessage>,
}

impl DocRoom {
    fn new(
        callback: UnboundedSender<DocRoomCallbackMessage>,
        id: u64,
        name: String,
        doc: Doc,
        recv: UnboundedReceiver<DocCommandMessage>,
    ) -> Self {
        let doc = Document::new(name, doc);
        Self {
            id,
            callback,
            doc,
            recv,
        }
    }

    pub fn create(
        principal: &Principal,
        doc: Doc,
        callback: UnboundedSender<DocRoomCallbackMessage>,
    ) -> DocRoomCommand {
        let id = principal.get_id();
        let name = principal.get_doc_name().to_owned();

        let (outgoing, incoming) = unbounded_channel();

        tokio::spawn(async move {
            let mut docroom = Self::new(callback, id, name, doc, incoming);

            docroom.run().await;
        });

        DocRoomCommand::new(outgoing)
    }

    async fn run(&mut self) {
        let mut timer = interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                msg = self.recv.recv() => {
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

        self.callback_store();
        if let Err(_err) = self.callback.send(DocRoomCallbackMessage::Destory(self.id)) {
            // TODO log error
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
                println!("callback store encode state failed, ${err}");
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
        }
    }
}
