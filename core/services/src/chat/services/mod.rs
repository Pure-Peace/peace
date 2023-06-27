pub mod channel;
pub mod chat;
pub mod traits;

use bytes::Bytes;
pub use channel::*;
pub use chat::*;
use peace_pb::chat::ChatMessageTarget;
pub use traits::*;

use super::Platform;

pub enum Subjects {
    Message,
}

impl Subjects {
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Message => "chat.message",
        }
    }
}

impl ToString for Subjects {
    #[inline]
    fn to_string(&self) -> String {
        self.as_str().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender_id: i32,
    pub message: String,
    pub platform: Platform,
    pub target: ChatMessageTarget,
}

impl ChatMessage {
    #[inline]
    pub fn to_bytes(&self) -> Result<Bytes, serde_json::Error> {
        Ok(Bytes::from(self.to_vec()?))
    }

    #[inline]
    pub fn to_vec(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    #[inline]
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
