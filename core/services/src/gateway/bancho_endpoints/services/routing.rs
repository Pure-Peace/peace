use super::traits::{
    BanchoRoutingService, DynBanchoHandlerService, DynBanchoRoutingService,
};
use crate::gateway::bancho_endpoints::{
    extractors::{BanchoClientToken, BanchoClientVersion},
    BanchoHttpError,
};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use peace_pb::bancho_state_rpc::UserQuery;
use std::{net::IpAddr, sync::Arc};

pub struct BanchoRoutingServiceImpl {
    bancho_handler_service: DynBanchoHandlerService,
}

impl BanchoRoutingServiceImpl {
    pub fn new(bancho_handler_service: DynBanchoHandlerService) -> Self {
        Self { bancho_handler_service }
    }

    pub fn into_service(self) -> DynBanchoRoutingService {
        Arc::new(self) as DynBanchoRoutingService
    }
}

#[async_trait]
impl BanchoRoutingService for BanchoRoutingServiceImpl {
    async fn bancho_get(&self) -> Response {
        tools::pkg_metadata!().into_response()
    }

    async fn bancho_post(
        &self,
        session_id: Option<BanchoClientToken>,
        version: Option<BanchoClientVersion>,
        ip: IpAddr,
        body: Vec<u8>,
    ) -> Result<Response, BanchoHttpError> {
        if session_id.is_none() {
            return self
                .bancho_handler_service
                .bancho_login(body, ip, version)
                .await
                .map_err(BanchoHttpError::LoginFailed);
        }

        let session_id = session_id.unwrap();

        let user_id = self
            .bancho_handler_service
            .check_user_session(UserQuery::SessionId(session_id.to_owned()))
            .await?;

        self.bancho_handler_service
            .bancho_post_responder(user_id, session_id, body)
            .await
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
        "".into_response()
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
