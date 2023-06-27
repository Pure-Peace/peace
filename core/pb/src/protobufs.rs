#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]

macro_rules! proto {
    ($package: tt) => {
        include!(concat!("../generated/", $package, ".rs"));
    };
}

macro_rules! descriptor {
    ($package: tt) => {
        include_bytes!(concat!("../generated/", $package, ".bin"))
    };
}

pub mod base {
    proto!("peace.base");

    pub const LOGS_DESCRIPTOR_SET: &[u8] = descriptor!("peace.base.descriptor");
}

pub mod logs {
    proto!("peace.frame.logs");

    pub const LOGS_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.frame.logs.descriptor");
}

pub mod bancho {
    proto!("peace.services.bancho");

    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.bancho.descriptor");
}

pub mod bancho_state {
    proto!("peace.services.bancho_state");

    pub const BANCHO_STATE_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.bancho_state.descriptor");

    use self::{
        raw_bancho_packet_target::TargetType, raw_user_query::QueryType,
    };
    use crate::ConvertError;
    use bitmask_enum::bitmask;
    use serde::{Deserialize, Serialize};
    use std::{error::Error, fmt, str::FromStr};
    use tools::{DecodingError, Ulid};

    #[bitmask(i32)]
    pub enum UserSessionFields {
        SessionId,
        UserId,
        Username,
        UsernameUnicode,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum UserQuery {
        SessionId(Ulid),
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
    }

    impl RawUserQuery {
        #[inline]
        pub fn into_user_query(self) -> Result<UserQuery, ConvertError> {
            self.try_into()
        }
    }

    impl TryFrom<RawUserQuery> for UserQuery {
        type Error = ConvertError;

        fn try_from(raw: RawUserQuery) -> Result<Self, Self::Error> {
            match raw.query_type() {
                QueryType::SessionId => Ok(Self::SessionId(Ulid::from_str(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?.as_str(),
                )?)),
                QueryType::UserId => Ok(Self::UserId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)?,
                )),
                QueryType::Username => Ok(Self::Username(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
                QueryType::UsernameUnicode => Ok(Self::UsernameUnicode(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
            }
        }
    }

    impl From<UserQuery> for RawUserQuery {
        fn from(query: UserQuery) -> Self {
            match query {
                UserQuery::SessionId(session_id) => RawUserQuery {
                    query_type: QueryType::SessionId as i32,
                    int_val: None,
                    string_val: Some(session_id.to_string()),
                },
                UserQuery::UserId(user_id) => RawUserQuery {
                    query_type: QueryType::UserId as i32,
                    int_val: Some(user_id),
                    string_val: None,
                },
                UserQuery::Username(username) => RawUserQuery {
                    query_type: QueryType::Username as i32,
                    int_val: None,
                    string_val: Some(username),
                },
                UserQuery::UsernameUnicode(username_unicode) => RawUserQuery {
                    query_type: QueryType::UsernameUnicode as i32,
                    int_val: None,
                    string_val: Some(username_unicode),
                },
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum BanchoPacketTarget {
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
        SessionId(Ulid),
        Channel(String),
    }

    impl BanchoPacketTarget {
        #[inline]
        pub fn into_user_query(self) -> Result<UserQuery, ConvertError> {
            self.try_into()
        }
    }

    impl RawBanchoPacketTarget {
        #[inline]
        pub fn into_packet_target(
            self,
        ) -> Result<BanchoPacketTarget, ConvertError> {
            self.try_into()
        }
    }

    impl TryFrom<RawBanchoPacketTarget> for BanchoPacketTarget {
        type Error = ConvertError;

        fn try_from(raw: RawBanchoPacketTarget) -> Result<Self, Self::Error> {
            match raw.target_type() {
                TargetType::SessionId => Ok(Self::SessionId(Ulid::from_str(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?.as_str(),
                )?)),
                TargetType::UserId => Ok(Self::UserId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::Username => Ok(Self::Username(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::UsernameUnicode => Ok(Self::UsernameUnicode(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::Channel => Ok(Self::Channel(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
            }
        }
    }

    impl TryInto<UserQuery> for BanchoPacketTarget {
        type Error = ConvertError;

        fn try_into(self) -> Result<UserQuery, Self::Error> {
            Ok(match self {
                Self::SessionId(session_id) => UserQuery::SessionId(session_id),
                Self::UserId(user_id) => UserQuery::UserId(user_id),
                Self::Username(username) => UserQuery::Username(username),
                Self::UsernameUnicode(username_unicode) => {
                    UserQuery::UsernameUnicode(username_unicode)
                },
                Self::Channel(_) => {
                    return Err(ConvertError::FromChannelTarget)
                },
            })
        }
    }

    impl From<BanchoPacketTarget> for RawBanchoPacketTarget {
        fn from(target: BanchoPacketTarget) -> Self {
            match target {
                BanchoPacketTarget::SessionId(session_id) => Self {
                    target_type: TargetType::SessionId as i32,
                    int_val: None,
                    string_val: Some(session_id.to_string()),
                },
                BanchoPacketTarget::UserId(user_id) => Self {
                    target_type: TargetType::UserId as i32,
                    int_val: Some(user_id),
                    string_val: None,
                },
                BanchoPacketTarget::Username(username) => Self {
                    target_type: TargetType::Username as i32,
                    int_val: None,
                    string_val: Some(username),
                },
                BanchoPacketTarget::UsernameUnicode(username_unicode) => Self {
                    target_type: TargetType::UsernameUnicode as i32,
                    int_val: None,
                    string_val: Some(username_unicode),
                },
                BanchoPacketTarget::Channel(channel) => Self {
                    target_type: TargetType::Channel as i32,
                    int_val: None,
                    string_val: Some(channel),
                },
            }
        }
    }
}

pub mod chat {
    proto!("peace.services.chat");

    pub const CHAT_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.chat.descriptor");

    use serde::{Deserialize, Serialize};

    use self::{
        raw_channel_query::QueryType, raw_chat_message_target::TargetType,
    };
    use crate::ConvertError;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum ChannelQuery {
        ChannelId(u64),
        ChannelName(String),
    }

    impl RawChannelQuery {
        #[inline]
        pub fn into_channel_query(self) -> Result<ChannelQuery, ConvertError> {
            self.try_into()
        }
    }

    impl TryFrom<RawChannelQuery> for ChannelQuery {
        type Error = ConvertError;

        fn try_from(raw: RawChannelQuery) -> Result<Self, Self::Error> {
            match raw.query_type() {
                QueryType::ChannelId => Ok(Self::ChannelId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)?,
                )),
                QueryType::ChannelName => Ok(Self::ChannelName(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
            }
        }
    }

    impl From<ChannelQuery> for RawChannelQuery {
        fn from(query: ChannelQuery) -> Self {
            match query {
                ChannelQuery::ChannelId(channel_id) => Self {
                    query_type: QueryType::ChannelId as i32,
                    int_val: Some(channel_id),
                    string_val: None,
                },
                ChannelQuery::ChannelName(channel_name) => Self {
                    query_type: QueryType::ChannelName as i32,
                    int_val: None,
                    string_val: Some(channel_name),
                },
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ChatMessageTarget {
        ChannelId(u64),
        ChannelName(String),
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
    }

    impl RawChatMessageTarget {
        #[inline]
        pub fn into_message_target(
            self,
        ) -> Result<ChatMessageTarget, ConvertError> {
            self.try_into()
        }
    }

    impl TryFrom<RawChatMessageTarget> for ChatMessageTarget {
        type Error = ConvertError;

        fn try_from(raw: RawChatMessageTarget) -> Result<Self, Self::Error> {
            match raw.target_type() {
                TargetType::ChannelId => Ok(Self::ChannelId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::ChannelName => Ok(Self::ChannelName(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::UserId => Ok(Self::UserId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)? as i32,
                )),
                TargetType::Username => Ok(Self::Username(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
                TargetType::UsernameUnicode => Ok(Self::UsernameUnicode(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                )),
            }
        }
    }

    impl From<ChatMessageTarget> for RawChatMessageTarget {
        fn from(target: ChatMessageTarget) -> Self {
            match target {
                ChatMessageTarget::ChannelId(channel_id) => Self {
                    target_type: TargetType::ChannelId as i32,
                    int_val: Some(channel_id),
                    string_val: None,
                },
                ChatMessageTarget::ChannelName(channel_name) => Self {
                    target_type: TargetType::ChannelName as i32,
                    int_val: None,
                    string_val: Some(channel_name),
                },
                ChatMessageTarget::UserId(user_id) => Self {
                    target_type: TargetType::UserId as i32,
                    int_val: Some(user_id as u64),
                    string_val: None,
                },
                ChatMessageTarget::Username(username) => Self {
                    target_type: TargetType::Username as i32,
                    int_val: None,
                    string_val: Some(username),
                },
                ChatMessageTarget::UsernameUnicode(username_unicode) => Self {
                    target_type: TargetType::UsernameUnicode as i32,
                    int_val: None,
                    string_val: Some(username_unicode),
                },
            }
        }
    }
}

pub mod geoip {
    proto!("peace.services.geoip");

    pub const GEOIP_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.geoip.descriptor");
}

pub mod signature {
    proto!("peace.services.signature");

    pub const SIGNATURE_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.signature.descriptor");
}
