use std::error::Error;

use super::PacketContext;
use crate::{
    bancho::{BanchoService, ProcessBanchoPacketError},
    bancho_state::PresenceFilter,
    chat::SessionPlatform,
};
use bancho_packets::{ClientChangeAction, PayloadReader};
use num_traits::FromPrimitive;
use peace_pb::{
    bancho::*,
    chat::{ChannelQuery, JoinIntoChannelRequest, LeaveFromChannelRequest},
};

#[inline]
fn handing_err(err: impl Error) -> ProcessBanchoPacketError {
    ProcessBanchoPacketError::Anyhow(anyhow!("{err:?}"))
}

#[inline]
fn read_channel_name(
    payload: Option<&[u8]>,
) -> Result<String, ProcessBanchoPacketError> {
    let mut channel_name = PayloadReader::new(
        payload.ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<String>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    if channel_name.starts_with('#') {
        channel_name.remove(0);
    }

    Ok(channel_name)
}

#[inline]
pub async fn user_channel_join<'a>(
    PacketContext { session_id: _, user_id, packet, svc_local, svc_impl: _ }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    let channel_name = read_channel_name(packet.payload)?;

    svc_local
        .chat_service
        .join_into_channel(JoinIntoChannelRequest {
            channel_query: Some(ChannelQuery::ChannelName(channel_name).into()),
            user_id,
            platforms: [SessionPlatform::Bancho as i32].into(),
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

#[inline]
pub async fn user_channel_part<'a>(
    PacketContext { session_id: _, user_id, packet, svc_local, svc_impl: _ }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    let channel_name = read_channel_name(packet.payload)?;

    svc_local
        .chat_service
        .leave_from_channel(LeaveFromChannelRequest {
            channel_query: Some(ChannelQuery::ChannelName(channel_name).into()),
            user_id,
            platforms: [SessionPlatform::Bancho as i32].into(),
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_request_status_update<'a>(
    PacketContext { session_id, user_id: _, packet: _, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    svc_impl
        .request_status_update(RequestStatusUpdateRequest {
            session_id: session_id.to_owned(),
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_presence_request_all<'a>(
    PacketContext { session_id, user_id: _, packet: _, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    svc_impl
        .presence_request_all(PresenceRequestAllRequest {
            session_id: session_id.to_owned(),
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_stats_request<'a>(
    PacketContext { session_id, user_id: _, packet, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    let request_users = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<Vec<i32>>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    svc_impl
        .request_stats(StatsRequest {
            session_id: session_id.to_owned(),
            request_users,
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_change_action<'a>(
    PacketContext { session_id, user_id: _, packet, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
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

    svc_impl
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

    Ok(())
}

pub async fn user_receive_updates<'a>(
    PacketContext { session_id, user_id: _, packet, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
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

    svc_impl
        .receive_updates(ReceiveUpdatesRequest {
            session_id: session_id.to_owned(),
            presence_filter: presence_filter.val(),
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_toggle_block_non_friend_dms<'a>(
    PacketContext { session_id, user_id: _, packet, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    let toggle = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<i32>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?
        == 1;

    svc_impl
        .toggle_block_non_friend_dms(ToggleBlockNonFriendDmsRequest {
            session_id: session_id.to_owned(),
            toggle,
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_logout<'a>(
    PacketContext { session_id, user_id: _, packet: _, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    svc_impl
        .user_logout(UserLogoutRequest { session_id: session_id.to_owned() })
        .await
        .map_err(handing_err)?;

    Ok(())
}

pub async fn user_presence_request<'a>(
    PacketContext { session_id, user_id: _, packet, svc_local: _, svc_impl }: PacketContext<'a>,
) -> Result<(), ProcessBanchoPacketError> {
    let request_users = PayloadReader::new(
        packet
            .payload
            .ok_or(ProcessBanchoPacketError::PacketPayloadNotExists)?,
    )
    .read::<Vec<i32>>()
    .ok_or(ProcessBanchoPacketError::InvalidPacketPayload)?;

    svc_impl
        .request_presence(PresenceRequest {
            session_id: session_id.to_owned(),
            request_users,
        })
        .await
        .map_err(handing_err)?;

    Ok(())
}
