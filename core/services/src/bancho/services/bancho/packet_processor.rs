use crate::{
    bancho::{traits::*, ProcessBanchoPacketError},
    bancho_state::{BanchoStateService, PresenceFilter},
    chat::ChatService,
};
use async_trait::async_trait;
use bancho_packets::{
    BanchoMessage, ClientChangeAction, Packet, PayloadReader,
};
use num_traits::FromPrimitive;
use peace_pb::{
    bancho::*,
    bancho_state::UserQuery,
    chat::{
        ChannelQuery, ChatMessageTarget, JoinChannelRequest,
        LeaveChannelRequest, SendMessageRequest,
    },
};
use std::{error::Error, fmt::Debug};

#[derive(Clone)]
pub struct PacketProcessor<'a> {
    pub user_id: i32,
    pub packet: Packet<'a>,
    pub bancho_service: &'a (dyn BanchoService + Send + Sync),
    pub bancho_state_service: &'a (dyn BanchoStateService + Send + Sync),
    pub chat_service: &'a (dyn ChatService + Send + Sync),
}

impl<'a> Debug for PacketProcessor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketProcessor")
            .field("user_id", &self.user_id)
            .field("packet", &self.packet)
            .finish()
    }
}

#[inline]
pub fn read_channel_name(
    payload: Option<&[u8]>,
) -> Result<String, ProcessBanchoPacketError> {
    let channel_name = PayloadReader::new(
        payload.ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<String>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    Ok(channel_name)
}

#[inline]
pub fn read_chat_message(
    payload: Option<&[u8]>,
) -> Result<BanchoMessage, ProcessBanchoPacketError> {
    let message = PayloadReader::new(
        payload.ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<BanchoMessage>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    Ok(message)
}

#[async_trait]
impl<'a> ProcessSendPublicMessage for PacketProcessor<'a> {
    #[inline]
    async fn send_public_message(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        #[allow(unused_mut)]
        let mut chat_message = read_chat_message(self.packet.payload)?;

        match chat_message.target.as_str() {
            "#spectator" => {
                // TODO: spectator chat
                todo!("get user's current #spectator channel id")
            },
            "#multiplayer" => {
                // TODO: multiplayer chat
                todo!("get user's current #multiplayer channel id")
            },
            _ => {},
        };

        let request = SendMessageRequest {
            sender: Some(UserQuery::UserId(self.user_id).into()),
            message: chat_message.content,
            target: Some(
                ChatMessageTarget::Channel(ChannelQuery::ChannelName(
                    chat_message.target,
                ))
                .into(),
            ),
        };

        self.chat_service.send_message(request).await?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessSendPrivateMessage for PacketProcessor<'a> {
    #[inline]
    async fn send_private_message(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let chat_message = read_chat_message(self.packet.payload)?;

        let request = SendMessageRequest {
            sender: Some(UserQuery::UserId(self.user_id).into()),
            message: chat_message.content,
            target: Some(
                ChatMessageTarget::User(UserQuery::Username(
                    chat_message.target,
                ))
                .into(),
            ),
        };

        self.chat_service.send_message(request).await.map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserChannelJoin for PacketProcessor<'a> {
    #[inline]
    async fn user_channel_join(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let channel_name = read_channel_name(self.packet.payload)?;

        self.chat_service
            .join_channel(JoinChannelRequest {
                channel_query: Some(
                    ChannelQuery::ChannelName(channel_name).into(),
                ),
                user_query: Some(UserQuery::UserId(self.user_id).into()),
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted { packets: None })
    }
}

#[async_trait]
impl<'a> ProcessUserChannelPart for PacketProcessor<'a> {
    #[inline]
    async fn user_channel_part(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let channel_name = read_channel_name(self.packet.payload)?;

        self.chat_service
            .leave_channel(LeaveChannelRequest {
                channel_query: Some(
                    ChannelQuery::ChannelName(channel_name).into(),
                ),
                user_query: Some(UserQuery::UserId(self.user_id).into()),
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted { packets: None })
    }
}

#[async_trait]
impl<'a> ProcessUserRequestStatusUpdate for PacketProcessor<'a> {
    #[inline]
    async fn user_request_status_update(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.bancho_service
            .request_status_update(UserQuery::UserId(self.user_id))
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserPresenceRequestAll for PacketProcessor<'a> {
    #[inline]
    async fn user_presence_request_all(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.bancho_service
            .presence_request_all(UserQuery::UserId(self.user_id))
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserStatsRequest for PacketProcessor<'a> {
    #[inline]
    async fn user_stats_request(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let request_users = PayloadReader::new(
            self.packet
                .payload
                .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
        )
        .read::<Vec<i32>>()
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

        self.bancho_service
            .request_stats(StatsRequest {
                user_id: self.user_id,
                request_users,
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserChangeAction for PacketProcessor<'a> {
    #[inline]
    async fn user_change_action(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let ClientChangeAction {
            online_status,
            description,
            beatmap_md5,
            mods,
            mode,
            beatmap_id,
        } = PayloadReader::new(
            self.packet
                .payload
                .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
        )
        .read::<ClientChangeAction>()
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

        self.bancho_service
            .change_action(ChangeActionRequest {
                user_id: self.user_id,
                online_status: online_status as i32,
                description,
                beatmap_md5,
                mods,
                mode: mode as i32,
                beatmap_id,
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserReceiveUpdates for PacketProcessor<'a> {
    #[inline]
    async fn user_receive_updates(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let presence_filter = PresenceFilter::from_i32(
            PayloadReader::new(
                self.packet
                    .payload
                    .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
            )
            .read::<i32>()
            .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?,
        )
        .unwrap_or_default();

        self.bancho_service
            .receive_updates(ReceiveUpdatesRequest {
                user_id: self.user_id,
                presence_filter: presence_filter.val(),
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserToggleBlockNonFriendDms for PacketProcessor<'a> {
    #[inline]
    async fn user_toggle_block_non_friend_dms(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let toggle = PayloadReader::new(
            self.packet
                .payload
                .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
        )
        .read::<i32>()
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?
            == 1;

        self.bancho_service
            .toggle_block_non_friend_dms(ToggleBlockNonFriendDmsRequest {
                user_id: self.user_id,
                toggle,
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserLogout for PacketProcessor<'a> {
    #[inline]
    async fn user_logout(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.bancho_service
            .user_logout(UserQuery::UserId(self.user_id))
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl<'a> ProcessUserPresenceRequest for PacketProcessor<'a> {
    #[inline]
    async fn user_presence_request(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let request_users = PayloadReader::new(
            self.packet
                .payload
                .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
        )
        .read::<Vec<i32>>()
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

        self.bancho_service
            .request_presence(PresenceRequest {
                user_id: self.user_id,
                request_users,
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}
