use super::traits::{BanchoHandlerService, DynBanchoHandlerService};
use crate::{
    bancho::DynBanchoService,
    bancho_state::{BanchoStateError, DynBanchoStateService},
    gateway::bancho_endpoints::{
        extractors::BanchoClientVersion, parser, BanchoHttpError, LoginError,
    },
};
use async_trait::async_trait;
use bancho_packets::PacketReader;
use peace_pb::{
    bancho::*,
    bancho_state::{
        BanchoPacketTarget, DequeueBanchoPacketsRequest, UserQuery,
    },
};
use std::{net::IpAddr, sync::Arc};

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
    #[inline]
    async fn bancho_login(
        &self,
        body: Vec<u8>,
        client_ip: IpAddr,
        version: Option<BanchoClientVersion>,
    ) -> Result<LoginSuccess, LoginError> {
        if version.is_none() {
            return Err(LoginError::EmptyClientVersion);
        }

        let request = parser::parse_osu_login_request_body(body)?;
        if request.client_version != version.unwrap().as_str() {
            return Err(LoginError::MismatchedClientVersion);
        }

        Ok(self
            .bancho_service
            .login(client_ip, request)
            .await
            .map_err(LoginError::BanchoServiceError)?)
    }

    #[inline]
    async fn process_bancho_packets(
        &self,
        user_id: i32,
        session_id: String,
        body: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, BanchoHttpError> {
        if PacketReader::new(&body).next().is_none() {
            return Err(BanchoHttpError::InvalidBanchoPacket);
        }

        let HandleCompleted { packets } = self
            .bancho_service
            .batch_process_bancho_packets(BatchProcessBanchoPacketsRequest {
                session_id,
                user_id,
                packets: body,
            })
            .await?;

        return Ok(packets);
    }

    #[inline]
    async fn pull_bancho_packets(
        &self,
        target: BanchoPacketTarget,
    ) -> Option<Vec<u8>> {
        self.bancho_state_service
            .dequeue_bancho_packets(DequeueBanchoPacketsRequest {
                target: Some(target.into()),
            })
            .await
            .map(|resp| resp.data)
            .ok()
    }

    #[inline]
    async fn check_user_session(
        &self,
        query: UserQuery,
    ) -> Result<i32, BanchoHttpError> {
        Ok(self
            .bancho_state_service
            .check_user_session_exists(query)
            .await
            .map_err(|err| match err {
                BanchoStateError::SessionNotExists => {
                    BanchoHttpError::SessionNotExists(err)
                },
                _ => BanchoHttpError::BanchoStateError(err),
            })?
            .user_id)
    }
}
