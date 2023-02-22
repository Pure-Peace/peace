use super::{
    constants::{CHO_PROTOCOL, CHO_TOKEN},
    parser,
};
use crate::{BanchoRpc, BanchoStateRpc, Error};
use axum::response::{IntoResponse, Response};
use bancho_packets::{Packet, PacketId};
use peace_api::extractors::BanchoClientVersion;
use peace_pb::services::{
    bancho_rpc::LoginSuccess, bancho_state_rpc::UserQuery,
};
use std::net::IpAddr;
use tonic::Request;
use tools::tonic_utils::RpcRequest;

pub async fn bancho_login(
    bancho: &mut BanchoRpc,
    body: impl Into<Vec<u8>>,
    version: Option<BanchoClientVersion>,
    ip: IpAddr,
) -> Result<Response, Error> {
    if version.is_none() {
        return Err(Error::Login("empty client version".into()));
    }

    let data = parser::parse_osu_login_request_body(body.into())?;

    if data.client_version != version.unwrap().as_str() {
        return Err(Error::Login("mismatched client version".into()));
    }

    let req = RpcRequest::new(data).client_ip_header(ip);

    let LoginSuccess { session_id, packet } = bancho
        .login(req.to_request())
        .await
        .map_err(|err| Error::Login(err.message().into()))?
        .into_inner();

    Ok((
        [(CHO_TOKEN, session_id.as_str()), CHO_PROTOCOL],
        packet.unwrap_or("ok".into()),
    )
        .into_response())
}

pub async fn check_user_session(
    bancho_state: &mut BanchoStateRpc,
    query: UserQuery,
) -> Result<i32, Error> {
    Ok(bancho_state
        .check_user_session_exists(Request::new(query.into()))
        .await
        .map_err(|err| Error::Login(err.message().into()))?
        .into_inner()
        .user_id)
}

pub async fn bancho_packet_handle(
    session_id: &str,
    user_id: i32,
    packet: &Packet<'_>,
    bancho: &mut BanchoRpc,
) -> Result<Response, Error> {
    match packet.id {
        // Message
        PacketId::OSU_SEND_PUBLIC_MESSAGE => {
            todo!() // chat.send_public_message
        },
        PacketId::OSU_SEND_PRIVATE_MESSAGE => {
            todo!() // chat.send_private_message
        },
        // User
        PacketId::OSU_USER_REQUEST_STATUS_UPDATE => todo!(),
        PacketId::OSU_USER_PRESENCE_REQUEST_ALL => todo!(),
        PacketId::OSU_USER_STATS_REQUEST => todo!(),
        PacketId::OSU_USER_CHANGE_ACTION => todo!(),
        PacketId::OSU_USER_RECEIVE_UPDATES => todo!(),
        PacketId::OSU_USER_FRIEND_ADD => todo!(),
        PacketId::OSU_USER_FRIEND_REMOVE => todo!(),
        PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS => todo!(),
        PacketId::OSU_USER_CHANNEL_PART => todo!(),
        PacketId::OSU_USER_CHANNEL_JOIN => todo!(),
        PacketId::OSU_USER_LOGOUT => todo!(),
        PacketId::OSU_USER_SET_AWAY_MESSAGE => todo!(),
        PacketId::OSU_USER_PRESENCE_REQUEST => todo!(),
        // Spectate
        PacketId::OSU_SPECTATE_START => todo!(),
        PacketId::OSU_SPECTATE_STOP => todo!(),
        PacketId::OSU_SPECTATE_CANT => todo!(),
        PacketId::OSU_SPECTATE_FRAMES => todo!(),
        // Multiplayer
        PacketId::OSU_USER_PART_LOBBY => todo!(),
        PacketId::OSU_USER_JOIN_LOBBY => todo!(),
        PacketId::OSU_USER_PART_MATCH => todo!(),
        PacketId::OSU_USER_MATCH_READY => todo!(),
        PacketId::OSU_USER_CREATE_MATCH => todo!(),
        PacketId::OSU_USER_JOIN_MATCH => todo!(),
        PacketId::OSU_MATCH_START => todo!(),
        PacketId::OSU_MATCH_COMPLETE => todo!(),
        PacketId::OSU_MATCH_LOAD_COMPLETE => todo!(),
        PacketId::OSU_MATCH_NO_BEATMAP => todo!(),
        PacketId::OSU_MATCH_NOT_READY => todo!(),
        PacketId::OSU_MATCH_FAILED => todo!(),
        PacketId::OSU_MATCH_HAS_BEATMAP => todo!(),
        PacketId::OSU_MATCH_SKIP_REQUEST => todo!(),
        PacketId::OSU_MATCH_CHANGE_TEAM => todo!(),
        PacketId::OSU_MATCH_CHANGE_SLOT => todo!(),
        PacketId::OSU_MATCH_LOCK => todo!(),
        PacketId::OSU_MATCH_CHANGE_SETTINGS => todo!(),
        PacketId::OSU_MATCH_SCORE_UPDATE => todo!(),
        PacketId::OSU_MATCH_CHANGE_MODS => todo!(),
        PacketId::OSU_MATCH_TRANSFER_HOST => todo!(),
        PacketId::OSU_MATCH_INVITE => todo!(),
        PacketId::OSU_MATCH_CHANGE_PASSWORD => todo!(),
        // Tournament
        PacketId::OSU_TOURNAMENT_MATCH_INFO_REQUEST => todo!(),
        PacketId::OSU_TOURNAMENT_JOIN_MATCH_CHANNEL => todo!(),
        PacketId::OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL => todo!(),
        _ => {
            warn!("unhandled packet: {packet:?}")
        },
    }
    todo!()
}
