use y_octo::JwstCodecError;

#[derive(Debug, Clone, Copy)]
pub enum DocMessage {
    Step1,
    Step2,
    Update,
}

impl TryFrom<u64> for DocMessage {
    type Error = JwstCodecError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Step1),
            1 => Ok(Self::Step2),
            2 => Ok(Self::Update),
            _ => Err(JwstCodecError::InvalidStructType(
                "invalid doc message type",
            )),
        }
    }
}

impl From<DocMessage> for u64 {
    fn from(value: DocMessage) -> Self {
        match value {
            DocMessage::Step1 => 0,
            DocMessage::Step2 => 1,
            DocMessage::Update => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Sync,
    Awareness,
    Auth,
    QueryAwareness,
    SyncReply,
    Stateless,
    BroadcastStateless,
    Close,
    SyncStatus,
}

impl TryFrom<u64> for MessageType {
    type Error = JwstCodecError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sync),
            1 => Ok(Self::Awareness),
            2 => Ok(Self::Auth),
            3 => Ok(Self::QueryAwareness),
            4 => Ok(Self::SyncReply),
            5 => Ok(Self::Stateless),
            6 => Ok(Self::BroadcastStateless),
            7 => Ok(Self::Close),
            8 => Ok(Self::SyncStatus),
            _ => Err(JwstCodecError::InvalidStructType("invalid message type")),
        }
    }
}

impl From<MessageType> for u64 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Sync => 0,
            MessageType::Awareness => 1,
            MessageType::Auth => 2,
            MessageType::QueryAwareness => 3,
            MessageType::SyncReply => 4,
            MessageType::Stateless => 5,
            MessageType::BroadcastStateless => 6,
            MessageType::Close => 7,
            MessageType::SyncStatus => 8,
        }
    }
}
