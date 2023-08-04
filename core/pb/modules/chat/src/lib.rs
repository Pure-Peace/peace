#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    use pb_base as base;

    pub mod services {
        use pb_bancho_state as bancho_state;

        pub mod chat {
            include!("../../../generated/peace.services.chat.rs");

            pub const CHAT_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.chat.descriptor.bin"
            );

            use self::{
                raw_channel_query::QueryType,
                raw_chat_message_target::ChatTarget,
            };
            use pb_bancho_state::UserQuery;
            use peace_pb::ConvertError;
            use peace_unique_id::Ulid;
            use std::str::FromStr;

            #[derive(
                Debug,
                Clone,
                PartialEq,
                Eq,
                Hash,
                serde::Deserialize,
                serde::Serialize,
            )]
            pub enum ChannelQuery {
                ChannelId(u64),
                ChannelName(String),
            }

            impl RawChannelQuery {
                #[inline]
                pub fn into_channel_query(
                    self,
                ) -> Result<ChannelQuery, ConvertError> {
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
                            raw.string_val
                                .ok_or(ConvertError::InvalidParams)?,
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

            #[derive(
                Debug,
                Clone,
                PartialEq,
                Eq,
                Hash,
                serde::Deserialize,
                serde::Serialize,
            )]
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

                fn try_from(
                    raw: RawChatMessageTarget,
                ) -> Result<Self, Self::Error> {
                    match raw.target_type() {
                        ChatTarget::ChannelId => {
                            Ok(Self::Channel(ChannelQuery::ChannelId(
                                raw.int_val
                                    .ok_or(ConvertError::InvalidParams)?,
                            )))
                        },
                        ChatTarget::ChannelName => {
                            Ok(Self::Channel(ChannelQuery::ChannelName(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?,
                            )))
                        },
                        ChatTarget::SessionId => Ok(Self::User(
                            UserQuery::SessionId(Ulid::from_str(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?
                                    .as_str(),
                            )?),
                        )),
                        ChatTarget::UserId => {
                            Ok(Self::User(UserQuery::UserId(
                                raw.int_val
                                    .ok_or(ConvertError::InvalidParams)?
                                    as i32,
                            )))
                        },
                        ChatTarget::Username => {
                            Ok(Self::User(UserQuery::Username(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?,
                            )))
                        },
                        ChatTarget::UsernameUnicode => {
                            Ok(Self::User(UserQuery::UsernameUnicode(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?,
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
                                ChannelQuery::ChannelName(channel_name) => {
                                    Self {
                                        target_type: ChatTarget::ChannelName
                                            as i32,
                                        int_val: None,
                                        string_val: Some(channel_name),
                                    }
                                },
                            }
                        },
                        ChatMessageTarget::User(user_query) => match user_query
                        {
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
                            UserQuery::UsernameUnicode(username_unicode) => {
                                Self {
                                    target_type: ChatTarget::UsernameUnicode
                                        as i32,
                                    int_val: None,
                                    string_val: Some(username_unicode),
                                }
                            },
                        },
                    }
                }
            }
        }
    }
}

pub use peace::services::chat::*;
