use num_derive::{FromPrimitive, ToPrimitive};

pub mod lsg_reply;
pub mod task_reply;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum BdMessageType {
    LobbyServiceTaskReply = 1,
    LobbyServicePushMessage = 2,
    LsgServiceError = 3,
    LsgServiceConnectionId = 4,
    LsgServiceTaskReply = 5,
}
