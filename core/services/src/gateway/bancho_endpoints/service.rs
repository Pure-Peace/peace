use crate::bancho_state::DynBanchoStateService;

use super::{repository::DynBanchoGatewayRepository, Error};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use bancho_packets::PacketReader;
use peace_api::extractors::*;
use peace_pb::bancho_state_rpc::{
    BanchoPacketTarget, DequeueBanchoPacketsRequest, UserQuery,
};
use std::{net::IpAddr, sync::Arc};
use tonic::Request;

pub type DynBanchoGatewayService = Arc<dyn BanchoGatewayService + Send + Sync>;

#[async_trait]
pub trait BanchoGatewayService {
    async fn bancho_get(&self) -> Response;

    async fn bancho_post(
        &self,
        session_id: Option<BanchoClientToken>,
        version: Option<BanchoClientVersion>,
        ip: IpAddr,
        body: Vec<u8>,
    ) -> Result<Response, Error>;

    async fn get_screenshot(&self) -> Response;

    async fn download_beatmapset(&self, beatmapset_id: i32) -> Response;

    async fn client_register(&self) -> Response;

    async fn ask_peppy(&self) -> Response;

    async fn difficulty_rating(&self) -> Response;

    async fn osu_error(&self) -> Response;

    async fn osu_screenshot(&self) -> Response;

    async fn osu_getfriends(&self) -> Response;

    async fn osu_getbeatmapinfo(&self) -> Response;

    async fn osu_getfavourites(&self) -> Response;

    async fn osu_addfavourite(&self) -> Response;

    async fn lastfm(&self) -> Response;

    async fn osu_search(&self) -> Response;

    async fn osu_search_set(&self) -> Response;

    async fn osu_submit_modular_selector(&self) -> Response;

    async fn osu_getreplay(&self) -> Response;

    async fn osu_rate(&self) -> Response;

    async fn osu_osz2_getscores(&self) -> Response;

    async fn osu_comment(&self) -> Response;

    async fn osu_markasread(&self) -> Response;

    async fn osu_getseasonal(&self) -> Response;

    async fn bancho_connect(&self) -> Response;

    async fn check_updates(&self) -> Response;

    async fn update_beatmap(&self) -> Response;
}

pub struct BanchoGatewayServiceImpl {
    bancho_gateway_repository: DynBanchoGatewayRepository,
    bancho_state_service: DynBanchoStateService,
}

impl BanchoGatewayServiceImpl {
    pub fn new(
        bancho_gateway_repository: DynBanchoGatewayRepository,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self { bancho_gateway_repository, bancho_state_service }
    }

    pub fn into_service(self) -> DynBanchoGatewayService {
        Arc::new(self) as DynBanchoGatewayService
    }
}

#[async_trait]
impl BanchoGatewayService for BanchoGatewayServiceImpl {
    async fn bancho_get(&self) -> Response {
        tools::pkg_metadata!().into_response()
    }

    async fn bancho_post(
        &self,
        session_id: Option<BanchoClientToken>,
        version: Option<BanchoClientVersion>,
        ip: IpAddr,
        body: Vec<u8>,
    ) -> Result<Response, Error> {
        if session_id.is_none() {
            return self
                .bancho_gateway_repository
                .bancho_login(body, ip, version)
                .await;
        }

        let session_id = session_id.unwrap();
        let user_id = self
            .bancho_gateway_repository
            .check_user_session(UserQuery::SessionId(session_id.to_owned()))
            .await?;

        let mut reader = PacketReader::new(&body);

        while let Some(packet) = reader.next() {
            debug!(
            "bancho packet received: {packet:?} (<{user_id}> [{session_id}])"
        );

            if let Err(err) = self
                .bancho_gateway_repository
                .process_bancho_packet(&session_id, user_id, &packet)
                .await
            {
                error!("bancho packet ({packet:?}) handle err: {err:?} (<{user_id}> [{session_id}])")
            }
        }

        let packets = self
            .bancho_state_service
            .dequeue_bancho_packets(Request::new(DequeueBanchoPacketsRequest {
                target: Some(
                    BanchoPacketTarget::SessionId(session_id.to_owned()).into(),
                ),
            }))
            .await;

        if let Err(err) = packets {
            error!(
            "dequeue bancho packets err: {err:?} (<{user_id}> [{session_id}])"
        );
            return Ok("ok".into_response());
        }

        return Ok(packets.unwrap().into_inner().data.into_response());
    }

    async fn get_screenshot(&self) -> Response {
        unimplemented!()
    }

    async fn download_beatmapset(&self, _beatmapset_id: i32) -> Response {
        unimplemented!()
    }

    async fn client_register(&self) -> Response {
        unimplemented!()
    }

    async fn ask_peppy(&self) -> Response {
        unimplemented!()
    }

    async fn difficulty_rating(&self) -> Response {
        unimplemented!()
    }

    async fn osu_error(&self) -> Response {
        "ok".into_response()
    }

    async fn osu_screenshot(&self) -> Response {
        unimplemented!()
    }

    async fn osu_getfriends(&self) -> Response {
        unimplemented!()
    }

    async fn osu_getbeatmapinfo(&self) -> Response {
        unimplemented!()
    }

    async fn osu_getfavourites(&self) -> Response {
        unimplemented!()
    }

    async fn osu_addfavourite(&self) -> Response {
        unimplemented!()
    }

    async fn lastfm(&self) -> Response {
        "ok".into_response()
    }

    async fn osu_search(&self) -> Response {
        unimplemented!()
    }

    async fn osu_search_set(&self) -> Response {
        unimplemented!()
    }

    async fn osu_submit_modular_selector(&self) -> Response {
        unimplemented!()
    }

    async fn osu_getreplay(&self) -> Response {
        unimplemented!()
    }

    async fn osu_rate(&self) -> Response {
        unimplemented!()
    }

    async fn osu_osz2_getscores(&self) -> Response {
        unimplemented!()
    }

    async fn osu_comment(&self) -> Response {
        unimplemented!()
    }

    async fn osu_markasread(&self) -> Response {
        "ok".into_response()
    }

    async fn osu_getseasonal(&self) -> Response {
        "ok".into_response()
    }

    async fn bancho_connect(&self) -> Response {
        "ok".into_response()
    }

    async fn check_updates(&self) -> Response {
        "ok".into_response()
    }

    async fn update_beatmap(&self) -> Response {
        "ok".into_response()
    }
}
