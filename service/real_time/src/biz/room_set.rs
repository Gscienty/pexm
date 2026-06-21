use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex, RwLock,
};
use y_octo::{Doc, DocOptions};

use crate::biz::{
    principal::Principal,
    room::{
        cmd::DocRoomCommand,
        doc_room::{DocRoom, DocRoomCallbackMessage},
    },
};

pub struct RoomSet<M>
where
    M: Clone,
{
    callback_outgoing: UnboundedSender<DocRoomCallbackMessage>,
    callback_incoming: Arc<Mutex<UnboundedReceiver<DocRoomCallbackMessage>>>,
    rooms: Arc<RwLock<HashMap<u64, (DocRoomCommand, M)>>>,
    /// Content from rooms that have exited but not yet persisted to dal.
    /// Key = room id, Value = last known doc state.
    /// When a new room is built while content is pending, use this instead of
    /// reading from dal (which may be stale).
    pending_content: Arc<RwLock<HashMap<u64, Vec<u8>>>>,
}

impl<M> Default for RoomSet<M>
where
    M: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M> RoomSet<M>
where
    M: Clone,
{
    pub fn new() -> Self {
        let (callback_outgoing, callback_incoming) = unbounded_channel();
        let rooms = Arc::new(RwLock::new(HashMap::new()));
        let pending_content = Arc::new(RwLock::new(HashMap::new()));

        Self {
            callback_outgoing,
            callback_incoming: Arc::new(Mutex::new(callback_incoming)),
            rooms,
            pending_content,
        }
    }

    pub async fn recv_callback(&self) -> Option<DocRoomCallbackMessage> {
        self.callback_incoming.lock().await.recv().await
    }

    /// Remove room from the active map and stash its content as pending.
    /// Called when DocRoom exits (all connections gone), before persisting to dal.
    pub async fn pre_destory_room(&self, id: u64, content: Vec<u8>) {
        let mut rooms = self.rooms.write().await;
        rooms.remove(&id);
        drop(rooms);

        let mut pending = self.pending_content.write().await;
        pending.insert(id, content);
    }

    /// Clear pending content after it has been successfully persisted to dal.
    pub async fn clear_pending(&self, id: u64) {
        let mut pending = self.pending_content.write().await;
        pending.remove(&id);
    }

    pub async fn get_room_model(&self, id: u64) -> Option<M> {
        let rooms = self.rooms.read().await;
        rooms.get(&id).map(|(_, model)| model.clone())
    }

    pub async fn get_docroom(&self, principal: &Principal) -> Option<DocRoomCommand> {
        let rooms = self.rooms.read().await;
        let (cmd, _) = rooms.get(&principal.get_id())?;
        // If the DocRoom task has exited, the receiver is dropped and the
        // sender is closed. Treat this as "room not found" so the caller
        // will build a fresh room instead of getting a dead command.
        if cmd.is_closed() {
            return None;
        }
        Some(cmd.clone())
    }

    /// Try to take pending content for a room (used when building a new room
    /// before the old content has been persisted to dal).
    pub async fn take_pending_content(&self, id: u64) -> Option<Vec<u8>> {
        let mut pending = self.pending_content.write().await;
        pending.remove(&id)
    }

    pub async fn build_docroom(
        &self,
        principal: &Principal,
        model: M,
        doc_binary: &[u8],
    ) -> Result<DocRoomCommand, ()> {
        let mut doc = Doc::with_options(DocOptions::new().auto_gc(true));
        if let Err(_err) = doc.apply_update_from_binary_v1(doc_binary) {
            // TODO log
        }

        let mut rooms = self.rooms.write().await;
        let (cmd, _) = rooms.entry(principal.get_id()).or_insert_with(|| {
            (
                DocRoom::create(principal, doc, self.callback_outgoing.clone()),
                model,
            )
        });

        Ok(cmd.clone())
    }
}
