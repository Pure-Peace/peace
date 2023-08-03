use bancho_packets::PacketId;
use core_bancho_state::BanchoStateError;
use core_chat::ChatError;
use peace_domain::users::PasswordError;
use peace_pb::ConvertError;
use peace_repositories::GetUserError;
use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum ProcessBanchoPacketError {
    #[error("failed to process all bancho packets")]
    FailedToProcessAll,
    #[error("invalid packet id")]
    InvalidPacketId,
    #[error("packet payload not exists")]
    PacketPayloadNotExists,
    #[error("invalid packet payload")]
    InvalidPacketPayload,
    #[error("unhandled packet: {0:?}")]
    UnhandledPacket(PacketId),
    #[error(transparent)]
    BanchoServiceError(#[from] BanchoServiceError),
    #[error(transparent)]
    ChatError(#[from] ChatError),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for ProcessBanchoPacketError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum BanchoServiceError {
    #[error(transparent)]
    PasswordError(#[from] PasswordError),
    #[error(transparent)]
    UserNotExists(#[from] GetUserError),
    #[error(transparent)]
    BanchoStateError(#[from] BanchoStateError),
    #[error(transparent)]
    ChatError(#[from] ChatError),
    #[error(transparent)]
    ConvertError(#[from] ConvertError),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for BanchoServiceError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}
