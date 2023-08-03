#![allow(warnings)]
#![allow(clippy)]
#![allow(unknown_lints)]
#![allow(non_snake_case)]

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

#[cfg(feature = "base")]
pub mod base {
    proto!("peace.base");

    pub const LOGS_DESCRIPTOR_SET: &[u8] = descriptor!("peace.base.descriptor");
}

#[cfg(feature = "logs")]
pub mod logs {
    proto!("peace.frame.logs");

    pub const LOGS_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.frame.logs.descriptor");
}

#[cfg(feature = "bancho")]
pub mod bancho {
    proto!("peace.services.bancho");

    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.bancho.descriptor");
}

#[cfg(feature = "bancho_state")]
pub mod bancho_state {
    proto!("peace.services.bancho_state");

    pub const BANCHO_STATE_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.bancho_state.descriptor");

    use self::raw_user_query::QueryType;
    use crate::ConvertError;
    use bitmask_enum::bitmask;
    use peace_unique_id::{raw::DecodingError, Ulid};
    use std::{error::Error, fmt, str::FromStr};

    #[bitmask(i32)]
    pub enum UserSessionFields {
        SessionId,
        UserId,
        Username,
        UsernameUnicode,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum UserQuery {
        SessionId(Ulid),
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
    }

    impl Into<String> for UserQuery {
        fn into(self) -> String {
            match self {
                Self::SessionId(val) => val.to_string(),
                Self::UserId(val) => val.to_string(),
                Self::Username(val) => val,
                Self::UsernameUnicode(val) => val,
            }
        }
    }

    impl ToString for UserQuery {
        fn to_string(&self) -> String {
            match self {
                Self::SessionId(val) => val.to_string(),
                Self::UserId(val) => val.to_string(),
                Self::Username(val) => val.to_owned(),
                Self::UsernameUnicode(val) => val.to_owned(),
            }
        }
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
}

#[cfg(feature = "chat")]
pub mod chat {
    proto!("peace.services.chat");

    pub const CHAT_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.chat.descriptor");

    use self::{
        raw_channel_query::QueryType, raw_chat_message_target::ChatTarget,
    };
    use crate::{bancho_state::UserQuery, ConvertError};
    use peace_unique_id::Ulid;
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    #[derive(Debug, Clone)]
    pub enum ChatMessageTarget {
        Channel(ChannelQuery),
        User(UserQuery),
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
                ChatTarget::ChannelId => {
                    Ok(Self::Channel(ChannelQuery::ChannelId(
                        raw.int_val.ok_or(ConvertError::InvalidParams)?,
                    )))
                },
                ChatTarget::ChannelName => {
                    Ok(Self::Channel(ChannelQuery::ChannelName(
                        raw.string_val.ok_or(ConvertError::InvalidParams)?,
                    )))
                },
                ChatTarget::SessionId => {
                    Ok(Self::User(UserQuery::SessionId(Ulid::from_str(
                        raw.string_val
                            .ok_or(ConvertError::InvalidParams)?
                            .as_str(),
                    )?)))
                },
                ChatTarget::UserId => Ok(Self::User(UserQuery::UserId(
                    raw.int_val.ok_or(ConvertError::InvalidParams)? as i32,
                ))),
                ChatTarget::Username => Ok(Self::User(UserQuery::Username(
                    raw.string_val.ok_or(ConvertError::InvalidParams)?,
                ))),
                ChatTarget::UsernameUnicode => {
                    Ok(Self::User(UserQuery::UsernameUnicode(
                        raw.string_val.ok_or(ConvertError::InvalidParams)?,
                    )))
                },
            }
        }
    }

    impl From<ChatMessageTarget> for RawChatMessageTarget {
        fn from(target: ChatMessageTarget) -> Self {
            match target {
                ChatMessageTarget::Channel(channel_query) => {
                    match channel_query {
                        ChannelQuery::ChannelId(channel_id) => Self {
                            target_type: ChatTarget::ChannelId as i32,
                            int_val: Some(channel_id),
                            string_val: None,
                        },
                        ChannelQuery::ChannelName(channel_name) => Self {
                            target_type: ChatTarget::ChannelName as i32,
                            int_val: None,
                            string_val: Some(channel_name),
                        },
                    }
                },
                ChatMessageTarget::User(user_query) => match user_query {
                    UserQuery::SessionId(session_id) => Self {
                        target_type: ChatTarget::SessionId as i32,
                        int_val: None,
                        string_val: Some(session_id.to_string()),
                    },
                    UserQuery::UserId(user_id) => Self {
                        target_type: ChatTarget::UserId as i32,
                        int_val: Some(user_id as u64),
                        string_val: None,
                    },
                    UserQuery::Username(username) => Self {
                        target_type: ChatTarget::Username as i32,
                        int_val: None,
                        string_val: Some(username),
                    },
                    UserQuery::UsernameUnicode(username_unicode) => Self {
                        target_type: ChatTarget::UsernameUnicode as i32,
                        int_val: None,
                        string_val: Some(username_unicode),
                    },
                },
            }
        }
    }
}

#[cfg(feature = "geoip")]
pub mod geoip {
    proto!("peace.services.geoip");

    pub const GEOIP_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.geoip.descriptor");
}

#[cfg(feature = "signature")]
pub mod signature {
    proto!("peace.services.signature");

    pub const SIGNATURE_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.signature.descriptor");
}
