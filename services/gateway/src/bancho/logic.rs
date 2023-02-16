use super::{constants::CHO_PROTOCOL, parser};
use crate::{utils::map_rpc_err, BanchoRpc, BanchoStateRpc};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use peace_api::{error::Error, extractors::BanchoClientVersion};
use peace_pb::services::{bancho_rpc::LoginReply, bancho_state_rpc::UserQuery};
use std::net::IpAddr;
use tonic::Request;
use tools::tonic_utils::RpcRequest;

pub async fn bancho_login(
    mut bancho: BanchoRpc,
    body: axum::body::Bytes,
    ip: IpAddr,
) -> Result<Response, Error> {
    let req =
        RpcRequest::new(parser::parse_osu_login_request_body(body.into())?)
            .client_ip_header(ip);

    let LoginReply { session_id, packet } =
        bancho.login(req.to_request()).await.map_err(map_rpc_err)?.into_inner();

    if session_id.is_none() {
        return Ok((
            StatusCode::UNAUTHORIZED,
            (
                [("cho-token", "failed"), CHO_PROTOCOL],
                packet.unwrap_or("failed".into()),
            ),
        )
            .into_response());
    }

    if packet.is_none() {
        return Err(Error::Internal);
    }

    Ok((
        [("cho-token", session_id.unwrap().as_str()), CHO_PROTOCOL],
        packet.unwrap_or("ok".into()),
    )
        .into_response())
}

pub async fn check_session(
    mut bancho_state: BanchoStateRpc,
}
