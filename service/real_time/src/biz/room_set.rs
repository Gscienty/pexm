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

        Self {
            callback_outgoing,
            callback_incoming: Arc::new(Mutex::new(callback_incoming)),
            rooms,
        }
    }

    pub async fn recv_callback(&self) -> Option<DocRoomCallbackMessage> {
        self.callback_incoming.lock().await.recv().await
    }

    pub async fn destory_room(&self, id: u64) {
        let mut rooms = self.rooms.write().await;
        rooms.remove(&id);
    }

    pub async fn get_room_model(&self, id: u64) -> Option<M> {
        let rooms = self.rooms.read().await;

        rooms.get(&id).map(|(_, model)| model.clone())
    }

    pub async fn get_docroom(&self, principal: &Principal) -> Option<DocRoomCommand> {
        let rooms = self.rooms.read().await;
        let (cmd, _) = rooms.get(&principal.get_id())?;
        Some(cmd.clone())
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
