use super::PacketContext;
use crate::{
    bancho::{traits::*, ProcessBanchoPacketError},
    bancho_state::PresenceFilter,
    chat::Platform,
};
use bancho_packets::{BanchoMessage, ClientChangeAction, PayloadReader};
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
use std::error::Error;

#[inline]
fn handing_err(err: impl Error) -> ProcessBanchoPacketError {
    ProcessBanchoPacketError::Anyhow(anyhow!("{err:?}"))
}

#[inline]
fn read_channel_name(
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
fn read_chat_message(
    payload: Option<&[u8]>,
) -> Result<BanchoMessage, ProcessBanchoPacketError> {
    let message = PayloadReader::new(
        payload.ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<BanchoMessage>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    Ok(message)
}

#[inline]
pub async fn send_public_message<'a>(
    PacketContext { user_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    #[allow(unused_mut)]
    let mut chat_message = read_chat_message(packet.payload)?;

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

    svc.chat_service
        .send_message(SendMessageRequest {
            sender_id: user_id,
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

#[inline]
pub async fn send_private_message<'a>(
    PacketContext { user_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let chat_message = read_chat_message(packet.payload)?;

    svc.chat_service
        .send_message(SendMessageRequest {
            sender_id: user_id,
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

#[inline]
pub async fn user_channel_join<'a>(
    PacketContext { user_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let channel_name = read_channel_name(packet.payload)?;

    let channel_info = svc
        .chat_service
        .add_user_into_channel(AddUserIntoChannelRequest {
            channel_query: Some(ChannelQuery::ChannelName(channel_name).into()),
            user_id,
            platforms: Some(ChatPlatforms {
                value: [Platform::Bancho as i32].into(),
            }),
        })
        .await
        .map_err(handing_err)?;

    svc.bancho_state_service
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

#[inline]
pub async fn user_channel_part<'a>(
    PacketContext { user_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let channel_name = read_channel_name(packet.payload)?;

    let channel_info = svc
        .chat_service
        .remove_user_platforms_from_channel(
            RemoveUserPlatformsFromChannelRequest {
                channel_query: Some(
                    ChannelQuery::ChannelName(channel_name).into(),
                ),
                user_id,
                platforms: Some(ChatPlatforms {
                    value: [Platform::Bancho as i32].into(),
                }),
            },
        )
        .await
        .map_err(handing_err)?;

    svc.bancho_state_service
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

#[inline]
pub async fn user_request_status_update<'a>(
    PacketContext { session_id, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    svc.request_status_update(RequestStatusUpdateRequest {
        session_id: session_id.to_owned(),
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_presence_request_all<'a>(
    PacketContext { session_id, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    svc.presence_request_all(PresenceRequestAllRequest {
        session_id: session_id.to_owned(),
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_stats_request<'a>(
    PacketContext { session_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let request_users = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<Vec<i32>>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    svc.request_stats(StatsRequest {
        session_id: session_id.to_owned(),
        request_users,
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_change_action<'a>(
    PacketContext { session_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
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
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<ClientChangeAction>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    svc.change_action(ChangeActionRequest {
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

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_receive_updates<'a>(
    PacketContext { session_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let presence_filter = PresenceFilter::from_i32(
        PayloadReader::new(
            packet
                .payload
                .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
        )
        .read::<i32>()
        .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?,
    )
    .unwrap_or_default();

    svc.receive_updates(ReceiveUpdatesRequest {
        session_id: session_id.to_owned(),
        presence_filter: presence_filter.val(),
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_toggle_block_non_friend_dms<'a>(
    PacketContext { session_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let toggle = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<i32>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)? ==
        1;

    svc.toggle_block_non_friend_dms(ToggleBlockNonFriendDmsRequest {
        session_id: session_id.to_owned(),
        toggle,
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_logout<'a>(
    PacketContext { session_id, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    svc.user_logout(UserLogoutRequest { session_id: session_id.to_owned() })
        .await
        .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}

#[inline]
pub async fn user_presence_request<'a>(
    PacketContext { session_id, packet, svc, .. }: PacketContext<'a>,
) -> Result<HandleCompleted, ProcessBanchoPacketError> {
    let request_users = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<Vec<i32>>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    svc.request_presence(PresenceRequest {
        session_id: session_id.to_owned(),
        request_users,
    })
    .await
    .map_err(handing_err)?;

    Ok(HandleCompleted::default())
}
