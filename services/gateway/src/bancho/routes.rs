use super::logic;
use crate::{BanchoRpc, BanchoStateRpc, Error};
use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    Extension,
};
use bancho_packets::PacketReader;
use peace_api::extractors::{
    BanchoClientToken, BanchoClientVersion, BanchoRequestBody, ClientIp,
};
use peace_pb::services::bancho_state_rpc::{
    BanchoPacketTarget, DequeueBanchoPacketsRequest, UserQuery,
};

/// Bancho get handler
#[utoipa::path(
    get,
    path = "/",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho get handler"),
    )
)]
pub async fn bancho_get() -> Response {
    tools::pkg_metadata!().into_response()
}

/// Bancho post handler
#[utoipa::path(
    post,
    path = "/",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho post handler", body = [String]),
    )
)]
pub async fn bancho_post(
    session_id: Option<BanchoClientToken>,
    version: Option<BanchoClientVersion>,
    ClientIp(ip): ClientIp,
    Extension(mut bancho): Extension<BanchoRpc>,
    Extension(mut bancho_state): Extension<BanchoStateRpc>,
    BanchoRequestBody(body): BanchoRequestBody,
) -> Result<Response, Error> {
    if session_id.is_none() {
        return logic::bancho_login(&mut bancho, body, version, ip).await;
    }

    let session_id = session_id.unwrap();
    let user_id = logic::check_user_session(
        &mut bancho_state,
        UserQuery::SessionId(session_id.to_owned()),
    )
    .await?;

    let mut reader = PacketReader::new(&body);

    while let Some(packet) = reader.next() {
        debug!(
            "bancho packet received: {packet:?} (<{user_id}> [{session_id}])"
        );

        if let Err(err) = logic::bancho_packet_handle(
            &session_id,
            user_id,
            &packet,
            &mut bancho,
        )
        .await
        {
            error!("bancho packet ({packet:?}) handle err: {err:?} (<{user_id}> [{session_id}])")
        }
    }

    let packets = bancho_state
        .dequeue_bancho_packets(DequeueBanchoPacketsRequest {
            target: Some(
                BanchoPacketTarget::SessionId(session_id.to_owned()).into(),
            ),
        })
        .await;

    if let Err(err) = packets {
        error!(
            "dequeue bancho packets err: {err:?} (<{user_id}> [{session_id}])"
        );
        return Ok("ok".into_response());
    }

    return Ok(packets.unwrap().into_inner().data.into_response());
}

/// Bancho get_screenshot
#[utoipa::path(
    get,
    path = "/ss/{screenshot}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho get_screenshot"),
    )
)]
pub async fn get_screenshot() -> Response {
    unimplemented!()
}

/// Bancho download_beatmapset
#[utoipa::path(
    get,
    path = "/d/{beatmapset_id}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho download_beatmapset"),
    )
)]
pub async fn download_beatmapset(Path(beatmapset_id): Path<i32>) -> Response {
    unimplemented!()
}

/// Bancho client_register
#[utoipa::path(
    post,
    path = "/users",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho client_register", body = [String]),
    )
)]
pub async fn client_register() -> Response {
    unimplemented!()
}

/// Bancho ask_peppy
#[utoipa::path(
    get,
    path = "/p/doyoureallywanttoaskpeppy",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho ask_peppy"),
    )
)]
pub async fn ask_peppy() -> Response {
    unimplemented!()
}

/// Bancho difficulty_rating
#[utoipa::path(
    get,
    path = "/difficulty-rating",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho difficulty_rating"),
    )
)]
pub async fn difficulty_rating() -> Response {
    unimplemented!()
}

/// Bancho osu_error
#[utoipa::path(
    post,
    path = "/web/osu-error.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_error", body = [String]),
    )
)]
pub async fn osu_error() -> Response {
    "ok".into_response()
}

/// Bancho osu_screenshot
#[utoipa::path(
    post,
    path = "/web/osu-screenshot.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_screenshot", body = [String]),
    )
)]
pub async fn osu_screenshot() -> Response {
    unimplemented!()
}

/// Bancho osu_getfriends
#[utoipa::path(
    get,
    path = "/web/osu-getfriends.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getfriends"),
    )
)]
pub async fn osu_getfriends() -> Response {
    unimplemented!()
}

/// Bancho osu_getbeatmapinfo
#[utoipa::path(
    get,
    path = "/web/osu-getbeatmapinfo.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getbeatmapinfo"),
    )
)]
pub async fn osu_getbeatmapinfo() -> Response {
    unimplemented!()
}

/// Bancho osu_getfavourites
#[utoipa::path(
    get,
    path = "/web/osu-getfavourites.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getfavourites"),
    )
)]
pub async fn osu_getfavourites() -> Response {
    unimplemented!()
}

/// Bancho osu_addfavourite
#[utoipa::path(
    get,
    path = "/web/osu-addfavourite.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_addfavourite"),
    )
)]
pub async fn osu_addfavourite() -> Response {
    unimplemented!()
}

/// Bancho lastfm
#[utoipa::path(
    get,
    path = "/web/osu-lastfm.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho lastfm"),
    )
)]
pub async fn lastfm() -> Response {
    "ok".into_response()
}

/// Bancho osu_search
#[utoipa::path(
    get,
    path = "/web/osu-search.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_search"),
    )
)]
pub async fn osu_search() -> Response {
    unimplemented!()
}

/// Bancho osu_search_set
#[utoipa::path(
    get,
    path = "/web/osu-search-set.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_search_set"),
    )
)]
pub async fn osu_search_set() -> Response {
    unimplemented!()
}

/// Bancho osu_submit_modular_selector
#[utoipa::path(
    post,
    path = "/web/osu-submit-modular-selector.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_submit_modular_selector", body = [String]),
    )
)]
pub async fn osu_submit_modular_selector() -> Response {
    unimplemented!()
}

/// Bancho osu_getreplay
#[utoipa::path(
    get,
    path = "/web/osu-getreplay.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getreplay"),
    )
)]
pub async fn osu_getreplay() -> Response {
    unimplemented!()
}

/// Bancho osu_rate
#[utoipa::path(
    get,
    path = "/web/osu-rate.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_rate"),
    )
)]
pub async fn osu_rate() -> Response {
    unimplemented!()
}

/// Bancho osu_osz2_getscores
#[utoipa::path(
    get,
    path = "/web/osu-osz2-getscores.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_osz2_getscores"),
    )
)]
pub async fn osu_osz2_getscores() -> Response {
    unimplemented!()
}

/// Bancho osu_comment
#[utoipa::path(
    post,
    path = "/web/osu-comment.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_comment", body = [String]),
    )
)]
pub async fn osu_comment() -> Response {
    unimplemented!()
}

/// Bancho osu_markasread
#[utoipa::path(
    get,
    path = "/web/osu-markasread.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_markasread"),
    )
)]
pub async fn osu_markasread() -> Response {
    "ok".into_response()
}

/// Bancho osu_getseasonal
#[utoipa::path(
    get,
    path = "/web/osu-getseasonal.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho osu_getseasonal"),
    )
)]
pub async fn osu_getseasonal() -> Response {
    "ok".into_response()
}

/// Bancho bancho_connect
#[utoipa::path(
    get,
    path = "/web/bancho_connect.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho bancho_connect"),
    )
)]
pub async fn bancho_connect() -> Response {
    "ok".into_response()
}

/// Bancho check_updates
#[utoipa::path(
    get,
    path = "/web/check-updates.php",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho check_updates"),
    )
)]
pub async fn check_updates() -> Response {
    "ok".into_response()
}

/// Bancho update_beatmap
#[utoipa::path(
    get,
    path = "/web/maps/{beatmap_file_name}",
    tag = "bancho",
    responses(
        (status = 200, description = "Bancho update_beatmap"),
    )
)]
pub async fn update_beatmap() -> Response {
    "ok".into_response()
}

pub async fn test() -> Response {
    "ok".into_response()
}
