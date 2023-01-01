use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use peace_api::{
    error::{map_err, Error},
    extractors::{ClientIp, OsuClientBody, OsuToken, OsuVersion},
};
use peace_pb::services::bancho::{
    bancho_rpc_client::BanchoRpcClient, LoginReply,
};
use tonic::{transport::Channel, Request};

use super::{
    constants::{CHO_PROTOCOL, CLIENT_IP_HEADER},
    parser,
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
    osu_token: Option<OsuToken>,
    OsuVersion(osu_version): OsuVersion,
    ClientIp(ip): ClientIp,
    State(mut bancho): State<BanchoRpcClient<Channel>>,
    OsuClientBody(body): OsuClientBody,
) -> Result<Response, Error> {
    if osu_token.is_none() {
        let mut req =
            Request::new(parser::parse_osu_login_request_body(body.into())?);

        req.metadata_mut()
            .insert(CLIENT_IP_HEADER, ip.to_string().parse().map_err(map_err)?);

        let LoginReply { token, packet } = bancho
            .login(req)
            .await
            .map_err(|err| {
                debug!("login rpc call failed with: {}", err);
                Error::Anyhow(anyhow!("{}", err.message()))
            })?
            .into_inner();

        if let Some(token) = token {
            return Ok((
                [("cho-token", token.as_str()), CHO_PROTOCOL],
                packet.unwrap_or("ok".into()),
            )
                .into_response());
        }

        return Ok((
            [("cho-token", "failed"), CHO_PROTOCOL],
            packet.unwrap_or("failed".into()),
        )
            .into_response());
    }

    /* println!("{:?} {} {}", osu_token, osu_version, ip);
    println!("{:?}", body); */

    Ok("ok".into_response())
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
    unimplemented!()
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
    unimplemented!()
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
