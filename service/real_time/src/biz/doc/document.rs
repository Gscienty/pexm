use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::sync::Mutex;
use y_octo::{Awareness, Doc, JwstCodecResult, StateVector};

use crate::biz::{
    principal::Principal,
    protocol::handler::{handle_message, handle_query_awareness},
};

use super::context::DocumentContext;

pub struct Document {
    pub(super) name: String,
    pub(super) doc: Doc,
    pub(super) awareness: Awareness,
    pub(super) version: AtomicU64,
    pub(super) stored_version: AtomicU64,

    pub(super) connections: HashMap<u64, Arc<Mutex<SplitSink<WebSocket, Message>>>>,
}

impl Document {
    pub fn new(name: String, doc: Doc) -> Self {
        Self {
            name,
            doc,
            awareness: Awareness::new(0),

            version: AtomicU64::new(0),
            stored_version: AtomicU64::new(0),

            connections: HashMap::new(),
        }
    }

    pub(crate) async fn connect(
        &mut self,
        principal: &Principal,
        connection: SplitSink<WebSocket, Message>,
    ) -> JwstCodecResult<()> {
        let connection = Arc::new(Mutex::new(connection));

        let ctx = DocumentContext::new(self, connection.clone());
        handle_query_awareness(&ctx).await?;

        self.connections
            .insert(principal.get_connection_id(), connection);
        Ok(())
    }

    pub(crate) fn disconnect(&mut self, id: u64) {
        self.connections.remove(&id);
    }

    pub async fn handle_message(
        &mut self,
        principal: &Principal,
        message: &[u8],
    ) -> JwstCodecResult<()> {
        let connection =
            if let Some(connection) = self.connections.get(&principal.get_connection_id()) {
                connection.clone()
            } else {
                return Ok(());
            };

        let mut ctx = DocumentContext::new(self, connection);
        let modified = handle_message(&mut ctx, message).await?;

        if ctx.is_closed() {
            self.connections.remove(&principal.get_connection_id());
        }

        if modified {
            self.version.fetch_add(1, Ordering::Acquire);
        }

        Ok(())
    }

    pub fn destory(&mut self) {
        self.doc.unsubscribe_all();
    }

    pub(crate) fn is_connection_empty(&self) -> bool {
        self.connections.is_empty()
    }

    pub(crate) fn encode_state(&self) -> JwstCodecResult<Vec<u8>> {
        self.doc.encode_state_as_update_v1(&StateVector::default())
    }

    pub(crate) fn should_store(&self) -> bool {
        self.stored_version.load(Ordering::Acquire) != self.version.load(Ordering::Acquire)
    }

    pub(crate) fn stored_notice(&self) {
        self.stored_version
            .store(self.version.load(Ordering::Acquire), Ordering::Release)
    }
}
