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

    const CONVERT_PANIC: &str = "This should never happen, please check that the value is passed correctly.";

    #[derive(Debug, Clone)]
    pub enum UserQuery {
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
        SessionId(String),
    }

    impl From<RawUserQuery> for UserQuery {
        fn from(raw: RawUserQuery) -> UserQuery {
            match raw.query_type() {
                QueryType::UserId =>
                    Self::UserId(raw.int_val.expect(CONVERT_PANIC)),
                QueryType::Username =>
                    Self::Username(raw.string_val.expect(CONVERT_PANIC)),
                QueryType::UsernameUnicode =>
                    Self::UsernameUnicode(raw.string_val.expect(CONVERT_PANIC)),
                QueryType::SessionId =>
                    Self::SessionId(raw.string_val.expect(CONVERT_PANIC)),
            }
        }
    }

    impl From<UserQuery> for RawUserQuery {
        fn from(query: UserQuery) -> RawUserQuery {
            match query {
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
                UserQuery::SessionId(session_id) => RawUserQuery {
                    query_type: QueryType::SessionId as i32,
                    int_val: None,
                    string_val: Some(session_id),
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
                TargetType::UserId =>
                    Self::UserId(raw.int_val.expect(CONVERT_PANIC)),
                TargetType::Username =>
                    Self::Username(raw.string_val.expect(CONVERT_PANIC)),
                TargetType::UsernameUnicode =>
                    Self::UsernameUnicode(raw.string_val.expect(CONVERT_PANIC)),
                TargetType::SessionId =>
                    Self::SessionId(raw.string_val.expect(CONVERT_PANIC)),
                TargetType::Channel =>
                    Self::Channel(raw.string_val.expect(CONVERT_PANIC)),
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
