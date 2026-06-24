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
    /// Uses std::sync::Mutex for synchronous access — the DocRoom writes
    /// to this before dropping its channel receiver, so there is no async
    /// gap between "room exited" and "content available in pending".
    pending_content: Arc<std::sync::Mutex<HashMap<u64, Vec<u8>>>>,
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
        let pending_content = Arc::new(std::sync::Mutex::new(HashMap::new()));

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

    /// Remove a closed room from the active map (if it is still the same
    /// closed room — i.e. hasn't been replaced by a new room).
    pub async fn remove_closed_room(&self, id: u64) {
        let mut rooms = self.rooms.write().await;
        if let Some((cmd, _)) = rooms.get(&id) {
            if cmd.is_closed() || cmd.has_exit_content() {
                rooms.remove(&id);
            }
        }
    }

    /// Read pending content without removing it.
    /// Used by the PreDestory handler to persist to DAL while keeping the
    /// content available for concurrent `get_content` calls.
    pub fn get_pending_content(&self, id: u64) -> Option<Vec<u8>> {
        self.pending_content.lock().unwrap().get(&id).cloned()
    }

    /// Clear pending content after it has been successfully persisted to dal.
    pub fn clear_pending(&self, id: u64) {
        self.pending_content.lock().unwrap().remove(&id);
    }

    pub async fn get_room_model(&self, id: u64) -> Option<M> {
        let rooms = self.rooms.read().await;
        rooms.get(&id).map(|(_, model)| model.clone())
    }

    pub async fn get_docroom(&self, principal: &Principal) -> Option<DocRoomCommand> {
        let rooms = self.rooms.read().await;
        let (cmd, _) = rooms.get(&principal.get_id())?;
        // If the DocRoom task has exited or is exiting, treat this as
        // "room not found" so the caller will build a fresh room instead
        // of getting a dead command.
        if cmd.is_closed() || cmd.has_exit_content() {
            return None;
        }
        Some(cmd.clone())
    }

    /// Try to take pending content for a room (used when building a new room
    /// before the old content has been persisted to dal).
    pub fn take_pending_content(&self, id: u64) -> Option<Vec<u8>> {
        self.pending_content.lock().unwrap().remove(&id)
    }

    /// Get the current document content for a room.
    /// Checks: active room (exit cache + channel) → pending content → None.
    /// Does NOT hold the rooms lock across await points.
    pub async fn get_content(&self, id: u64) -> Option<Vec<u8>> {
        // 1. Check active room. Clone the cmd first, then release the lock
        //    before calling get_content (which may await).
        let cmd = {
            let rooms = self.rooms.read().await;
            rooms.get(&id).map(|(cmd, _)| cmd.clone())
        };
        if let Some(cmd) = cmd {
            if let Some(content) = cmd.get_content().await {
                return Some(content);
            }
        }

        // 2. Check pending content (room removed from active map but
        //    content not yet persisted to dal).
        if let Some(content) = self.pending_content.lock().unwrap().get(&id).cloned() {
            return Some(content);
        }

        None
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
        match rooms.entry(principal.get_id()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let (cmd, _) = e.get();
                if !cmd.is_closed() && !cmd.has_exit_content() {
                    return Ok(cmd.clone());
                }
                // Room exited — replace with a fresh one.
                let new_cmd = DocRoom::create(
                    principal,
                    doc,
                    self.callback_outgoing.clone(),
                    self.pending_content.clone(),
                );
                e.insert((new_cmd.clone(), model));
                Ok(new_cmd)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let new_cmd = DocRoom::create(
                    principal,
                    doc,
                    self.callback_outgoing.clone(),
                    self.pending_content.clone(),
                );
                e.insert((new_cmd.clone(), model));
                Ok(new_cmd)
            }
        }
    }
}
