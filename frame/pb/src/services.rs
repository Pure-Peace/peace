#[cfg(feature = "bancho")]
pub mod bancho_rpc {
    tonic::include_proto!("peace.services.bancho");

    #[cfg(feature = "reflection")]
    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_descriptor");
}

#[cfg(feature = "peace_db")]
pub mod peace_db_rpc {
    tonic::include_proto!("peace.services.db.peace");

    #[cfg(feature = "reflection")]
    pub const PEACE_DB_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("peace_db_descriptor");
}

#[cfg(feature = "bancho_state")]
pub mod bancho_state_rpc {
    tonic::include_proto!("peace.services.bancho.state");

    #[cfg(feature = "reflection")]
    pub const BANCHO_STATE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_state_descriptor");

    use self::{
        raw_bancho_packet_target::TargetType, raw_user_query::QueryType,
    };
    use bitmask_enum::bitmask;

    const CONVERT_PANIC: &str = "This should never happen, please check that the value is passed correctly.";

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
        fn from(raw: RawUserQuery) -> UserQuery {
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
        fn from(query: UserQuery) -> RawUserQuery {
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
        fn from(raw: RawBanchoPacketTarget) -> BanchoPacketTarget {
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

    impl From<BanchoPacketTarget> for RawBanchoPacketTarget {
        fn from(target: BanchoPacketTarget) -> RawBanchoPacketTarget {
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

#[cfg(feature = "chat")]
pub mod chat_rpc {
    tonic::include_proto!("peace.services.chat");

    #[cfg(feature = "reflection")]
    pub const CHAT_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("chat_descriptor");
}
