use super::{constants::CHO_PROTOCOL, parser};
use crate::{BanchoRpc, BanchoStateRpc, Error};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use peace_api::extractors::BanchoClientVersion;
use peace_pb::services::{bancho_rpc::LoginReply, bancho_state_rpc::UserQuery};
use std::net::IpAddr;
use tonic::Request;
use tools::tonic_utils::RpcRequest;

pub async fn bancho_login(
    mut bancho: BanchoRpc,
    body: axum::body::Bytes,
    version: Option<BanchoClientVersion>,
    ip: IpAddr,
) -> Result<Response, Error> {
    if version.is_none() {
        return Err(Error::Login("invalid client version".into()));
    }

    let req =
        RpcRequest::new(parser::parse_osu_login_request_body(body.into())?)
            .client_ip_header(ip);

    let LoginReply { session_id, packet } = bancho
        .login(req.to_request())
        .await
        .map_err(|err| Error::Login(err.message().into()))?
        .into_inner();

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

    Ok((
        [("cho-token", session_id.unwrap().as_str()), CHO_PROTOCOL],
        packet.unwrap_or("ok".into()),
    )
        .into_response())
}

pub async fn check_session(
    mut bancho_state: BanchoStateRpc,
    query: UserQuery,
) -> Result<(), Error> {
    bancho_state
        .check_user_session_exists(Request::new(query.into()))
        .await
        .map_err(|err| Error::Login(err.message().into()))?;

    Ok(())
}
