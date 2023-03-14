use super::traits::{BanchoHandlerService, DynBanchoHandlerService};
use crate::{
    bancho::DynBanchoService,
    bancho_state::{BanchoStateError, DynBanchoStateService, PresenceFilter},
    gateway::bancho_endpoints::{
        extractors::{BanchoClientToken, BanchoClientVersion},
        parser, BanchoHttpError, LoginError, CHO_PROTOCOL, CHO_TOKEN,
    },
};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use bancho_packets::{
    ClientChangeAction, Packet, PacketId, PacketReader, PayloadReader,
};
use num_traits::FromPrimitive;
use peace_pb::{
    bancho::*,
    bancho_state::{
        BanchoPacketTarget, DequeueBanchoPacketsRequest, UserQuery,
    },
};
use std::{error::Error, net::IpAddr, sync::Arc, time::Instant};

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
            return Err(LoginError::EmptyClientVersion)
        }

        let request = parser::parse_osu_login_request_body(body)?;
        if request.client_version != version.unwrap().as_str() {
            return Err(LoginError::MismatchedClientVersion)
        }

        let LoginSuccess { session_id, user_id, mut packets } = self
            .bancho_service
            .login(client_ip, request)
            .await
            .map_err(LoginError::BanchoServiceError)?;

        match self
            .bancho_state_service
            .dequeue_bancho_packets(DequeueBanchoPacketsRequest {
                target: Some(BanchoPacketTarget::UserId(user_id).into()),
            })
            .await
        {
            Ok(dequeued_packets) if !dequeued_packets.data.is_empty() => {
                packets.extend(dequeued_packets.data);
            },
            Ok(_) => {},
            Err(err) => {
                error!("Failed to dequeue bancho packets: {err}")
            },
        };

        Ok(([(CHO_TOKEN, session_id.as_str()), CHO_PROTOCOL], packets)
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
            info!(target: "bancho_packet_handling", "packet received: {packet}");
            let start = Instant::now();

            self.process_bancho_packet(&session_id, user_id, packet)
                .await
                .unwrap_or_else(|err| {
                    error!(target: "bancho_packet_handling", "{err:?} (<{user_id}> [{session_id}])")
                });

            info!(target: "bancho_packet_handling", "packet handled in {:?}", start.elapsed());
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
        return Ok(packets.data.into_response())
    }

    async fn check_user_session(
        &self,
        query: UserQuery,
    ) -> Result<i32, BanchoHttpError> {
        Ok(self
            .bancho_state_service
            .check_user_session_exists(query)
            .await
            .map_err(|err| match err {
                BanchoStateError::SessionNotExists =>
                    BanchoHttpError::SessionNotExists(err),
                _ => BanchoHttpError::BanchoStateError(err),
            })?
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
            PacketId::OSU_USER_CHANNEL_PART => {
                todo!() // channel
            },
            PacketId::OSU_USER_CHANNEL_JOIN => {
                todo!() // channel
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
            PacketId::OSU_USER_PRESENCE_REQUEST_ALL => {
                self.bancho_service
                    .presence_request_all(PresenceRequestAllRequest {
                        session_id: session_id.to_owned(),
                    })
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_STATS_REQUEST => {
                let request_users = PayloadReader::new(
                    packet
                        .payload
                        .ok_or(BanchoHttpError::PacketPayloadNotExists)?,
                )
                .read::<Vec<i32>>()
                .ok_or(BanchoHttpError::InvalidPacketPayload)?;

                self.bancho_service
                    .request_stats(StatsRequest {
                        session_id: session_id.to_owned(),
                        request_users,
                    })
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_CHANGE_ACTION => {
                let ClientChangeAction {
                    online_status,
                    description,
                    beatmap_md5,
                    mods,
                    mode,
                    beatmap_id,
                } = PayloadReader::new(
                    packet
                        .payload
                        .ok_or(BanchoHttpError::PacketPayloadNotExists)?,
                )
                .read::<ClientChangeAction>()
                .ok_or(BanchoHttpError::InvalidPacketPayload)?;

                self.bancho_service
                    .change_action(ChangeActionRequest {
                        session_id: session_id.to_owned(),
                        online_status: online_status as i32,
                        description,
                        beatmap_md5,
                        mods,
                        mode: mode as i32,
                        beatmap_id,
                    })
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_RECEIVE_UPDATES => {
                let presence_filter = PresenceFilter::from_i32(
                    PayloadReader::new(
                        packet
                            .payload
                            .ok_or(BanchoHttpError::PacketPayloadNotExists)?,
                    )
                    .read::<i32>()
                    .ok_or(BanchoHttpError::InvalidPacketPayload)?,
                )
                .ok_or(BanchoHttpError::InvalidParams)?;

                self.bancho_service
                    .receive_updates(ReceiveUpdatesRequest {
                        session_id: session_id.to_owned(),
                        presence_filter: presence_filter.val(),
                    })
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_FRIEND_ADD => todo!(),
            PacketId::OSU_USER_FRIEND_REMOVE => todo!(),
            PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS => {
                let toggle = PayloadReader::new(
                    packet
                        .payload
                        .ok_or(BanchoHttpError::PacketPayloadNotExists)?,
                )
                .read::<i32>()
                .ok_or(BanchoHttpError::InvalidParams)? ==
                    1;

                self.bancho_service
                    .toggle_block_non_friend_dms(
                        ToggleBlockNonFriendDmsRequest {
                            session_id: session_id.to_owned(),
                            toggle,
                        },
                    )
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_LOGOUT => {
                self.bancho_service
                    .user_logout(UserLogoutRequest {
                        session_id: session_id.to_owned(),
                    })
                    .await
                    .map_err(handing_err)?;
            },
            PacketId::OSU_USER_SET_AWAY_MESSAGE => todo!(),
            PacketId::OSU_USER_PRESENCE_REQUEST => {
                let request_users = PayloadReader::new(
                    packet
                        .payload
                        .ok_or(BanchoHttpError::PacketPayloadNotExists)?,
                )
                .read::<Vec<i32>>()
                .ok_or(BanchoHttpError::InvalidPacketPayload)?;

                self.bancho_service
                    .request_presence(PresenceRequest {
                        session_id: session_id.to_owned(),
                        request_users,
                    })
                    .await
                    .map_err(handing_err)?;
            },
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
