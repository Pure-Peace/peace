use super::traits::{BanchoHandlerService, DynBanchoHandlerService};
use crate::{
    bancho::DynBanchoService,
    bancho_state::DynBanchoStateService,
    gateway::bancho_endpoints::{
        extractors::{BanchoClientToken, BanchoClientVersion},
        parser, BanchoHttpError, LoginError, CHO_PROTOCOL, CHO_TOKEN,
    },
};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use bancho_packets::{Packet, PacketId, PacketReader};
use peace_pb::{
    bancho_rpc::{LoginSuccess, RequestStatusUpdateRequest},
    bancho_state_rpc::{
        BanchoPacketTarget, DequeueBanchoPacketsRequest, UserQuery,
    },
};
use std::{error::Error, net::IpAddr, sync::Arc};

#[derive(Clone)]
pub struct BanchoHandlerServiceImpl {
    bancho_service: DynBanchoService,
    bancho_state_service: DynBanchoStateService,
}

impl BanchoHandlerServiceImpl {
    pub fn new(
        bancho_service: DynBanchoService,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self { bancho_service, bancho_state_service }
    }

    pub fn into_service(self) -> DynBanchoHandlerService {
        Arc::new(self) as DynBanchoHandlerService
    }
}

#[async_trait]
impl BanchoHandlerService for BanchoHandlerServiceImpl {
    async fn bancho_login(
        &self,
        body: Vec<u8>,
        client_ip: IpAddr,
        version: Option<BanchoClientVersion>,
    ) -> Result<Response, LoginError> {
        if version.is_none() {
            return Err(LoginError::EmptyClientVersion);
        }

        let request = parser::parse_osu_login_request_body(body)?;
        if request.client_version != version.unwrap().as_str() {
            return Err(LoginError::MismatchedClientVersion);
        }

        let LoginSuccess { session_id, packet } = self
            .bancho_service
            .login(client_ip, request)
            .await
            .map_err(LoginError::BanchoServiceError)?;

        Ok((
            [(CHO_TOKEN, session_id.as_str()), CHO_PROTOCOL],
            packet.unwrap_or("ok".into()),
        )
            .into_response())
    }

    async fn bancho_post_responder(
        &self,
        user_id: i32,
        BanchoClientToken(session_id): BanchoClientToken,
        body: Vec<u8>,
    ) -> Result<Response, BanchoHttpError> {
        let mut reader = PacketReader::new(&body);

        while let Some(packet) = reader.next() {
            debug!("bancho packet received: {packet:?} (<{user_id}> [{session_id}])");

            self.process_bancho_packet(&session_id, user_id, packet)
                .await
                .unwrap_or_else(|err| {
                    error!("{err} (<{user_id}> [{session_id}])")
                });
        }

        let packets = self
            .bancho_state_service
            .dequeue_bancho_packets(DequeueBanchoPacketsRequest {
                target: Some(
                    BanchoPacketTarget::SessionId(session_id.to_owned()).into(),
                ),
            })
            .await
            .map_err(|err| {
                let err = BanchoHttpError::DequeuePakcetsError(err);
                error!("{err}");
                err
            })?;
        return Ok(packets.data.into_response());
    }

    async fn check_user_session(
        &self,
        query: UserQuery,
    ) -> Result<i32, BanchoHttpError> {
        Ok(self
            .bancho_state_service
            .check_user_session_exists(query)
            .await
            .map_err(BanchoHttpError::SessionNotExists)?
            .user_id)
    }

    async fn process_bancho_packet(
        &self,
        session_id: &str,
        _user_id: i32,
        packet: Packet<'_>,
    ) -> Result<(), BanchoHttpError> {
        fn handing_err(err: impl Error) -> BanchoHttpError {
            BanchoHttpError::PacketHandlingError(anyhow!("{err:?}"))
        }

        match packet.id {
            PacketId::OSU_PING => {},
            // Message
            PacketId::OSU_SEND_PUBLIC_MESSAGE => {
                todo!() // chat.send_public_message
            },
            PacketId::OSU_SEND_PRIVATE_MESSAGE => {
                todo!() // chat.send_private_message
            },
            // User
            PacketId::OSU_USER_REQUEST_STATUS_UPDATE => {
                self.bancho_service
                    .request_status_update(RequestStatusUpdateRequest {
                        session_id: session_id.to_owned(),
                    })
                    .await
                    .map_err(handing_err)?;
            },
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
            _ => return Err(BanchoHttpError::UnhandledPacket(packet.id)),
        };

        Ok(())
    }
}
