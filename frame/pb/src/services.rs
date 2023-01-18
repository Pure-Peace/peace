#[cfg(feature = "bancho")]
pub mod bancho {
    tonic::include_proto!("peace.services.bancho");

    #[cfg(feature = "reflection")]
    pub const BANCHO_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_descriptor");
}

#[cfg(feature = "peace_db")]
pub mod peace_db {
    tonic::include_proto!("peace.services.db.peace");

    #[cfg(feature = "reflection")]
    pub const PEACE_DB_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("peace_db_descriptor");
}

#[cfg(feature = "bancho_state")]
pub mod bancho_state {
    tonic::include_proto!("peace.services.bancho.state");

    #[cfg(feature = "reflection")]
    pub const BANCHO_STATE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bancho_state_descriptor");

    use self::{
        raw_bancho_packet_target::TargetType, raw_user_query::QueryType,
    };
    use crate::ConvertError;

    #[derive(Debug, Clone)]
    pub enum UserQuery {
        UserId(i32),
        Username(String),
        UsernameUnicode(String),
        SessionId(String),
    }

    impl TryFrom<RawUserQuery> for UserQuery {
        type Error = ConvertError;

        fn try_from(raw: RawUserQuery) -> Result<UserQuery, ConvertError> {
            Ok(match raw.query_type() {
                QueryType::UserId => Self::UserId(i32::from_le_bytes(
                    raw.key.try_into().map_err(|_| {
                        ConvertError("Invalid user_id bytes".into())
                    })?,
                )),
                QueryType::Username => Self::Username(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
                QueryType::UsernameUnicode => Self::UsernameUnicode(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
                QueryType::SessionId => Self::SessionId(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
            })
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

    impl TryFrom<RawBanchoPacketTarget> for BanchoPacketTarget {
        type Error = ConvertError;

        fn try_from(
            raw: RawBanchoPacketTarget,
        ) -> Result<BanchoPacketTarget, ConvertError> {
            Ok(match raw.target_type() {
                TargetType::UserId => Self::UserId(i32::from_le_bytes(
                    raw.key.try_into().map_err(|_| {
                        ConvertError("Invalid user_id bytes".into())
                    })?,
                )),
                TargetType::Username => Self::Username(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
                TargetType::UsernameUnicode => Self::UsernameUnicode(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
                TargetType::SessionId => Self::SessionId(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
                TargetType::Channel => Self::Channel(
                    String::from_utf8(raw.key).map_err(ConvertError::new)?,
                ),
            })
        }
    }
}
