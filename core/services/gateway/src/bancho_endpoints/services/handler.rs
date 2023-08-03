use super::traits::{BanchoHandlerService, DynBanchoHandlerService};
use crate::bancho_endpoints::{extractors::BanchoClientVersion, *};
use async_trait::async_trait;
use axum::response::{IntoResponse, Response};
use bancho_packets::PacketBuilder;
use bancho_packets::PacketReader;
use core_bancho::DynBanchoService;
use core_bancho_state::{BanchoStateError, DynBanchoStateService};
use core_chat::{ChatError, DynChatService};
use domain_bancho::BanchoClientToken;
use peace_pb::{
    bancho::*,
    bancho_state::{
        CheckUserTokenResponse, DequeueBanchoPacketsRequest, UserQuery,
    },
};
use std::{net::IpAddr, str::FromStr, sync::Arc};
use tools::lazy_init;

#[derive(Clone)]
pub struct BanchoHandlerServiceImpl {
    pub bancho_service: DynBanchoService,
    pub bancho_state_service: DynBanchoStateService,
    pub chat_service: DynChatService,
}

impl BanchoHandlerServiceImpl {
    pub fn new(
        bancho_service: DynBanchoService,
        bancho_state_service: DynBanchoStateService,
        chat_service: DynChatService,
    ) -> Self {
        Self { bancho_service, bancho_state_service, chat_service }
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

        Ok(self.bancho_service.login(client_ip, request).await?)
    }

    #[inline]
    async fn handle_logged(
        &self,
        token: String,
        body: Vec<u8>,
    ) -> Result<Response, BanchoHttpError> {
        let token = BanchoClientToken::from_str(&token)
            .map_err(|_| BanchoHttpError::InvalidOsuTokenHeader)?;

        if !self.check_user_token(token.clone()).await? {
            return Err(BanchoStateError::SessionNotExists)?;
        }

        let BanchoClientToken { user_id, .. } = token;

        let mut builder = None::<PacketBuilder>;

        if let Some(extra_packets) =
            self.process_bancho_packets(user_id, body).await?
        {
            lazy_init!(builder => builder.extend(extra_packets), PacketBuilder::from(extra_packets))
        }

        if let Ok(extra_packets) =
            self.pull_bancho_packets(UserQuery::UserId(user_id)).await
        {
            lazy_init!(builder => builder.extend(extra_packets), PacketBuilder::from(extra_packets))
        }

        if let Ok(extra_packets) =
            self.pull_chat_packets(UserQuery::UserId(user_id)).await
        {
            lazy_init!(builder => builder.extend(extra_packets), PacketBuilder::from(extra_packets))
        }

        return Ok(builder
            .map(|b| b.build())
            .unwrap_or_default()
            .into_response());
    }

    #[inline]
    async fn handle_not_logged(
        &self,
        version: Option<BanchoClientVersion>,
        ip: IpAddr,
        body: Vec<u8>,
    ) -> Result<Response, BanchoHttpError> {
        let LoginSuccess { session_id, signature, user_id, mut packets } =
            self.bancho_login(body, ip, version).await?;

        if let Ok(p) =
            self.pull_bancho_packets(UserQuery::UserId(user_id)).await
        {
            packets.extend(p);
        }

        Ok((
            [
                (
                    CHO_TOKEN,
                    BanchoClientToken::encode(user_id, &session_id, &signature)
                        .as_str(),
                ),
                CHO_PROTOCOL,
            ],
            packets,
        )
            .into_response())
    }

    #[inline]
    async fn process_bancho_packets(
        &self,
        user_id: i32,
        body: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, BanchoHttpError> {
        if PacketReader::new(&body).next().is_none() {
            return Err(BanchoHttpError::InvalidBanchoPacket);
        }

        let HandleCompleted { packets } =
            self.bancho_service
                .batch_process_bancho_packets(
                    BatchProcessBanchoPacketsRequest { user_id, packets: body },
                )
                .await?;

        return Ok(packets);
    }

    #[inline]
    async fn pull_bancho_packets(
        &self,
        user_query: UserQuery,
    ) -> Result<Vec<u8>, BanchoStateError> {
        self.bancho_state_service
            .dequeue_bancho_packets(DequeueBanchoPacketsRequest {
                user_query: Some(user_query.into()),
            })
            .await
            .map(|resp| resp.data)
    }

    #[inline]
    async fn pull_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<Vec<u8>, ChatError> {
        self.chat_service
            .dequeue_chat_packets(query)
            .await
            .map(|resp| resp.data)
    }

    #[inline]
    async fn check_user_token(
        &self,
        token: BanchoClientToken,
    ) -> Result<bool, BanchoStateError> {
        let CheckUserTokenResponse { is_valid } =
            self.bancho_state_service.check_user_token(token).await?;

        Ok(is_valid)
    }
}
