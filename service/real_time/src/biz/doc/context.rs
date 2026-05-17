use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::Mutex;

use crate::biz::protocol::context::Context;

use super::document::Document;

pub(super) struct DocumentContext<'s> {
    document: &'s mut Document,
    connection: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    closed: bool,
}

impl<'s> DocumentContext<'s> {
    pub(super) fn new(
        document: &'s mut Document,
        connection: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) -> Self {
        Self {
            document,
            connection,
            closed: false,
        }
    }

    pub(super) const fn is_closed(&self) -> bool {
        self.closed
    }
}

impl<'s> Context for DocumentContext<'s> {
    async fn unicast(&self, msg: Vec<u8>) {
        let mut connection = self.connection.lock().await;
        if let Err(_err) = connection.send(Message::Binary(msg.into())).await {
            // TODO log err
        }
    }

    async fn broadcast(&mut self, msg: &[u8]) {
        let mut invalid_connections = Vec::new();
        for (&cid, connection) in self.document.connections.iter() {
            let mut connection = connection.lock().await;
            if let Err(_err) = connection
                .send(Message::binary(Bytes::from_owner(msg.to_owned())))
                .await
            {
                // TODO log err

                invalid_connections.push(cid);
            }
        }

        for &id in invalid_connections.iter() {
            self.document.disconnect(id);
        }
    }

    async fn close(&mut self) {
        let mut connection = self.connection.lock().await;
        if let Err(_err) = connection.close().await {
            // TODO log err
        }

        self.closed = true;
    }

    fn get_document(&self) -> &y_octo::Doc {
        &self.document.doc
    }

    fn get_awareness(&self) -> &y_octo::Awareness {
        &self.document.awareness
    }

    fn get_document_mut(&mut self) -> &mut y_octo::Doc {
        &mut self.document.doc
    }

    fn get_document_name(&self) -> &str {
        &self.document.name
    }

    fn get_awareness_mut(&mut self) -> &mut y_octo::Awareness {
        &mut self.document.awareness
    }
}
