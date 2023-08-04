#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    use pb_base as base;

    pub mod services {
        use pb_geoip as geoip;

        pub mod bancho_state {
            include!("../../../generated/peace.services.bancho_state.rs");

            pub const BANCHO_STATE_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.bancho_state.descriptor.bin"
            );

            use self::raw_user_query::QueryType;
            use bitmask_enum::bitmask;
            use peace_pb::ConvertError;
            use peace_unique_id::Ulid;
            use std::str::FromStr;

            #[bitmask(i32)]
            pub enum UserSessionFields {
                SessionId,
                UserId,
                Username,
                UsernameUnicode,
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
            pub enum UserQuery {
                SessionId(Ulid),
                UserId(i32),
                Username(String),
                UsernameUnicode(String),
            }

            #[allow(clippy::from_over_into)]
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
                pub fn into_user_query(
                    self,
                ) -> Result<UserQuery, ConvertError> {
                    self.try_into()
                }
            }

            impl TryFrom<RawUserQuery> for UserQuery {
                type Error = ConvertError;

                fn try_from(raw: RawUserQuery) -> Result<Self, Self::Error> {
                    match raw.query_type() {
                        QueryType::SessionId => {
                            Ok(Self::SessionId(Ulid::from_str(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?
                                    .as_str(),
                            )?))
                        },
                        QueryType::UserId => Ok(Self::UserId(
                            raw.int_val.ok_or(ConvertError::InvalidParams)?,
                        )),
                        QueryType::Username => Ok(Self::Username(
                            raw.string_val
                                .ok_or(ConvertError::InvalidParams)?,
                        )),
                        QueryType::UsernameUnicode => {
                            Ok(Self::UsernameUnicode(
                                raw.string_val
                                    .ok_or(ConvertError::InvalidParams)?,
                            ))
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
                        UserQuery::UsernameUnicode(username_unicode) => {
                            RawUserQuery {
                                query_type: QueryType::UsernameUnicode as i32,
                                int_val: None,
                                string_val: Some(username_unicode),
                            }
                        },
                    }
                }
            }
        }
    }
}

pub use peace::services::bancho_state::*;
