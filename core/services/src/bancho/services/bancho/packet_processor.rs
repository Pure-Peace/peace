use crate::{
    bancho::{traits::*, BanchoServiceImpl, ProcessBanchoPacketError},
    bancho_state::PresenceFilter,
    chat::Platform,
};
use async_trait::async_trait;
use bancho_packets::{
    BanchoMessage, ClientChangeAction, Packet, PayloadReader,
};
use num_traits::FromPrimitive;
use peace_pb::{
    bancho::*,
    bancho_state::ChannelUpdateNotifyRequest,
    chat::{
        AddUserIntoChannelRequest, ChannelQuery, ChatMessageTarget,
        ChatPlatform, ChatPlatforms, RemoveUserPlatformsFromChannelRequest,
        SendMessageRequest,
    },
};
use std::{error::Error, fmt::Debug};

#[derive(Clone)]
pub struct PacketProcessor<'a> {
    pub session_id: &'a str,
    pub user_id: i32,
    pub packet: Packet<'a>,
    pub service: &'a BanchoServiceImpl,
}

impl<'a> Debug for PacketProcessor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketProcessor")
            .field("session_id", &self.session_id)
            .field("user_id", &self.user_id)
            .field("packet", &self.packet)
            .finish()
    }
}

#[inline]
pub fn handing_err(err: impl Error) -> ProcessBanchoPacketError {
    ProcessBanchoPacketError::Anyhow(anyhow!("{err:?}"))
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
                todo!("spectator chat")
            },
            "#multiplayer" => {
                // TODO: multiplayer chat
                todo!("multiplayer chat")
            },
            _ => {},
        };

        self.service
            .chat_service
            .send_message(SendMessageRequest {
                sender_id: self.user_id,
                message: chat_message.content,
                target: Some(
                    ChatMessageTarget::ChannelName(chat_message.target).into(),
                ),
                platform: ChatPlatform::Bancho as i32,
            })
            .await
            .map_err(handing_err)?;

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

        self.service
            .chat_service
            .send_message(SendMessageRequest {
                sender_id: self.user_id,
                message: chat_message.content,
                target: Some(
                    ChatMessageTarget::Username(chat_message.target).into(),
                ),
                platform: ChatPlatform::Bancho as i32,
            })
            .await
            .map_err(handing_err)?;

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

        let channel_info = self
            .service
            .chat_service
            .add_user_into_channel(AddUserIntoChannelRequest {
                channel_query: Some(
                    ChannelQuery::ChannelName(channel_name).into(),
                ),
                user_id: self.user_id,
                platforms: Some(ChatPlatforms {
                    value: [Platform::Bancho as i32].into(),
                }),
            })
            .await
            .map_err(handing_err)?;

        self.service
            .bancho_state_service
            .channel_update_notify(ChannelUpdateNotifyRequest {
                notify_targets: None,
                channel_info: Some(channel_info.to_owned()),
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted {
            packets: Some(bancho_packets::server::ChannelJoin::pack(
                channel_info.name.into(),
            )),
        })
    }
}

#[async_trait]
impl<'a> ProcessUserChannelPart for PacketProcessor<'a> {
    #[inline]
    async fn user_channel_part(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let channel_name = read_channel_name(self.packet.payload)?;

        let channel_info = self
            .service
            .chat_service
            .remove_user_platforms_from_channel(
                RemoveUserPlatformsFromChannelRequest {
                    channel_query: Some(
                        ChannelQuery::ChannelName(channel_name).into(),
                    ),
                    user_id: self.user_id,
                    platforms: Some(ChatPlatforms {
                        value: [Platform::Bancho as i32].into(),
                    }),
                },
            )
            .await
            .map_err(handing_err)?;

        self.service
            .bancho_state_service
            .channel_update_notify(ChannelUpdateNotifyRequest {
                notify_targets: None,
                channel_info: Some(channel_info.to_owned()),
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted {
            packets: Some(bancho_packets::server::ChannelKick::pack(
                channel_info.name.into(),
            )),
        })
    }
}

#[async_trait]
impl<'a> ProcessUserRequestStatusUpdate for PacketProcessor<'a> {
    #[inline]
    async fn user_request_status_update(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.service
            .request_status_update(RequestStatusUpdateRequest {
                session_id: self.session_id.to_owned(),
            })
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
        self.service
            .presence_request_all(PresenceRequestAllRequest {
                session_id: self.session_id.to_owned(),
            })
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

        self.service
            .request_stats(StatsRequest {
                session_id: self.session_id.to_owned(),
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

        self.service
            .change_action(ChangeActionRequest {
                session_id: self.session_id.to_owned(),
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

        self.service
            .receive_updates(ReceiveUpdatesRequest {
                session_id: self.session_id.to_owned(),
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
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)? ==
            1;

        self.service
            .toggle_block_non_friend_dms(ToggleBlockNonFriendDmsRequest {
                session_id: self.session_id.to_owned(),
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
        self.service
            .user_logout(UserLogoutRequest {
                session_id: self.session_id.to_owned(),
            })
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

        self.service
            .request_presence(PresenceRequest {
                session_id: self.session_id.to_owned(),
                request_users,
            })
            .await
            .map_err(handing_err)?;

        Ok(HandleCompleted::default())
    }
}
