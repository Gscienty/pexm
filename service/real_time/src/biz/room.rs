use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub type OnConnBeforeHook = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
>;

pub type OnConnAfterHook = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

pub type OnLeaveHook = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

pub struct Pipeline {
    on_conn_before_hooks: Vec<OnConnBeforeHook>,
    on_conn_after_hooks: Vec<OnConnAfterHook>,
    on_leave_hooks: Vec<OnLeaveHook>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            on_conn_before_hooks: Vec::new(),
            on_conn_after_hooks: Vec::new(),
            on_leave_hooks: Vec::new(),
        }
    }

    pub fn add_on_conn_before<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync + 'static,
    {
        self.on_conn_before_hooks.push(Arc::new(hook));
    }

    pub fn add_on_conn_after<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        self.on_conn_after_hooks.push(Arc::new(hook));
    }

    pub fn add_on_leave<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        self.on_leave_hooks.push(Arc::new(hook));
    }

    pub async fn run_on_conn_before(&self, token: &str) -> bool {
        for hook in &self.on_conn_before_hooks {
            if !hook(token.to_string()).await {
                return false;
            }
        }
        true
    }

    pub async fn run_on_conn_after(&self, token: &str) {
        for hook in &self.on_conn_after_hooks {
            hook(token.to_string()).await;
        }
    }

    pub async fn run_on_leave(&self, token: &str) {
        for hook in &self.on_leave_hooks {
            hook(token.to_string()).await;
        }
    }
}

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(0);

fn next_conn_id() -> u64 {
    NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed)
}

struct Room {
    members: HashMap<u64, mpsc::UnboundedSender<String>>,
}

impl Room {
    fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    fn join(&mut self, tx: mpsc::UnboundedSender<String>) -> u64 {
        let id = next_conn_id();
        self.members.insert(id, tx);
        id
    }

    fn leave(&mut self, conn_id: u64) {
        self.members.remove(&conn_id);
    }

    fn broadcast(&mut self, msg: &str) {
        self.members
            .retain(|_, sender| sender.send(msg.to_string()).is_ok());
    }
}

pub struct RoomManager {
    rooms: Mutex<HashMap<String, Room>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
        }
    }

    pub async fn join_room(&self, room_id: &str, tx: mpsc::UnboundedSender<String>) -> u64 {
        let mut rooms = self.rooms.lock().await;
        rooms
            .entry(room_id.to_string())
            .or_insert_with(Room::new)
            .join(tx)
    }

    pub async fn leave_room(&self, room_id: &str, conn_id: u64) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.leave(conn_id);
        }
    }

    pub async fn broadcast(&self, room_id: &str, msg: &str) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.broadcast(msg);
        }
    }
}

pub struct DocSyncRoomManager {
    room_manager: RoomManager,
    pipeline: Pipeline,
}

impl DocSyncRoomManager {
    pub fn new() -> Self {
        Self {
            room_manager: RoomManager::new(),
            pipeline: Pipeline::new(),
        }
    }

    pub fn add_on_conn_before<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync + 'static,
    {
        self.pipeline.add_on_conn_before(hook);
    }

    pub fn add_on_conn_after<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        self.pipeline.add_on_conn_after(hook);
    }

    pub fn add_on_leave<F>(&mut self, hook: F)
    where
        F: Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        self.pipeline.add_on_leave(hook);
    }

    pub async fn try_join(&self, token: &str) -> Option<(u64, mpsc::UnboundedReceiver<String>)> {
        if !self.pipeline.run_on_conn_before(token).await {
            return None;
        }
        let (tx, rx) = mpsc::unbounded_channel();
        let conn_id = self.room_manager.join_room(token, tx).await;
        self.pipeline.run_on_conn_after(token).await;
        Some((conn_id, rx))
    }

    pub async fn leave(&self, room_id: &str, conn_id: u64) {
        self.pipeline.run_on_leave(room_id).await;
        self.room_manager.leave_room(room_id, conn_id).await;
    }

    pub async fn broadcast(&self, room_id: &str, msg: &str) {
        self.room_manager.broadcast(room_id, msg).await;
    }
}