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

const CONVERT_PANIC: &str = "This should never happen, please check that the value is passed correctly.";

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
    use crate::protobufs::CONVERT_PANIC;
    use bitmask_enum::bitmask;

    #[bitmask(i32)]
    pub enum UserSessionFields {
        SessionId,
        UserId,
        Username,
        UsernameUnicode,
    }

    #[derive(Debug, Clone)]
    pub enum UserQuery {
        SessionId(String),
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
    }

    impl From<RawUserQuery> for UserQuery {
        fn from(raw: RawUserQuery) -> Self {
            match raw.query_type() {
                QueryType::SessionId => {
                    Self::SessionId(raw.string_val.expect(CONVERT_PANIC))
                },
                QueryType::UserId => {
                    Self::UserId(raw.int_val.expect(CONVERT_PANIC))
                },
                QueryType::Username => {
                    Self::Username(raw.string_val.expect(CONVERT_PANIC))
                },
                QueryType::UsernameUnicode => {
                    Self::UsernameUnicode(raw.string_val.expect(CONVERT_PANIC))
                },
            }
        }
    }

    impl From<UserQuery> for RawUserQuery {
        fn from(query: UserQuery) -> Self {
            match query {
                UserQuery::SessionId(session_id) => RawUserQuery {
                    query_type: QueryType::SessionId as i32,
                    int_val: None,
                    string_val: Some(session_id),
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

    #[derive(Debug, Clone)]
    pub enum BanchoPacketTarget {
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
        SessionId(String),
        Channel(String),
    }

    impl From<RawBanchoPacketTarget> for BanchoPacketTarget {
        fn from(raw: RawBanchoPacketTarget) -> Self {
            match raw.target_type() {
                TargetType::SessionId => {
                    Self::SessionId(raw.string_val.expect(CONVERT_PANIC))
                },
                TargetType::UserId => {
                    Self::UserId(raw.int_val.expect(CONVERT_PANIC))
                },
                TargetType::Username => {
                    Self::Username(raw.string_val.expect(CONVERT_PANIC))
                },
                TargetType::UsernameUnicode => {
                    Self::UsernameUnicode(raw.string_val.expect(CONVERT_PANIC))
                },
                TargetType::Channel => {
                    Self::Channel(raw.string_val.expect(CONVERT_PANIC))
                },
            }
        }
    }

    impl TryInto<UserQuery> for BanchoPacketTarget {
        type Error = ();

        fn try_into(self) -> Result<UserQuery, Self::Error> {
            Ok(match self {
                Self::SessionId(session_id) => UserQuery::SessionId(session_id),
                Self::UserId(user_id) => UserQuery::UserId(user_id),
                Self::Username(username) => UserQuery::Username(username),
                Self::UsernameUnicode(username_unicode) => {
                    UserQuery::UsernameUnicode(username_unicode)
                },
                Self::Channel(_) => return Err(()),
            })
        }
    }

    impl From<BanchoPacketTarget> for RawBanchoPacketTarget {
        fn from(target: BanchoPacketTarget) -> Self {
            match target {
                BanchoPacketTarget::SessionId(session_id) => Self {
                    target_type: TargetType::SessionId as i32,
                    int_val: None,
                    string_val: Some(session_id),
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

    use self::raw_channel_query::QueryType;
    use crate::protobufs::CONVERT_PANIC;

    #[derive(Debug, Clone)]
    pub enum ChannelQuery {
        ChannelId(u64),
        ChannelName(String),
    }

    impl From<RawChannelQuery> for ChannelQuery {
        fn from(raw: RawChannelQuery) -> Self {
            match raw.query_type() {
                QueryType::ChannelId => {
                    Self::ChannelId(raw.int_val.expect(CONVERT_PANIC))
                },
                QueryType::ChannelName => {
                    Self::ChannelName(raw.string_val.expect(CONVERT_PANIC))
                },
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
}

pub mod geoip {
    proto!("peace.services.geoip");

    pub const GEOIP_DESCRIPTOR_SET: &[u8] =
        descriptor!("peace.services.geoip.descriptor");
}
