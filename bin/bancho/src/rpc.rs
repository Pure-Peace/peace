use bancho_packets::{Packet, PacketId};
use bancho_service::DynBanchoService;
use peace_pb::{bancho::*, bancho_state::RawUserQuery};
use peace_rpc::extensions::ClientIp;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct BanchoRpcImpl {
    pub bancho_service: DynBanchoService,
}

impl BanchoRpcImpl {
    pub fn new(bancho_service: DynBanchoService) -> Self {
        Self { bancho_service }
    }
}

#[tonic::async_trait]
impl bancho_rpc_server::BanchoRpc for BanchoRpcImpl {
    async fn batch_process_bancho_packets(
        &self,
        request: Request<BatchProcessBanchoPacketsRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .batch_process_bancho_packets(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn process_bancho_packet(
        &self,
        request: Request<ProcessBanchoPacketRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let ProcessBanchoPacketRequest { user_id, packet_id, payload } =
            request.into_inner();

        let packet = Packet {
            id: PacketId::try_from(packet_id)
                .map_err(|_| Status::internal("invalid packet id"))?,
            payload: payload.as_deref(),
        };

        let res =
            self.bancho_service.process_bancho_packet(user_id, packet).await?;

        Ok(Response::new(res))
    }

    async fn ping(
        &self,
        _: Request<PingRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self.bancho_service.ping().await?;

        Ok(Response::new(res))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, Status> {
        let client_ip = ClientIp::from_request(&request)?;

        let res = self
            .bancho_service
            .login(client_ip.into(), request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn request_status_update(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .request_status_update(
                raw_user_query.into_inner().into_user_query()?,
            )
            .await?;

        Ok(Response::new(res))
    }

    async fn presence_request_all(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .presence_request_all(
                raw_user_query.into_inner().into_user_query()?,
            )
            .await?;

        Ok(Response::new(res))
    }

    async fn request_stats(
        &self,
        request: Request<StatsRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res =
            self.bancho_service.request_stats(request.into_inner()).await?;

        Ok(Response::new(res))
    }

    async fn change_action(
        &self,
        request: Request<ChangeActionRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res =
            self.bancho_service.change_action(request.into_inner()).await?;

        Ok(Response::new(res))
    }

    async fn receive_updates(
        &self,
        request: Request<ReceiveUpdatesRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res =
            self.bancho_service.receive_updates(request.into_inner()).await?;

        Ok(Response::new(res))
    }

    async fn toggle_block_non_friend_dms(
        &self,
        request: Request<ToggleBlockNonFriendDmsRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .toggle_block_non_friend_dms(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn user_logout(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .user_logout(raw_user_query.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn request_presence(
        &self,
        request: Request<PresenceRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res =
            self.bancho_service.request_presence(request.into_inner()).await?;

        Ok(Response::new(res))
    }

    async fn spectate_stop(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .spectate_stop(raw_user_query.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn spectate_cant(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .spectate_cant(raw_user_query.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn lobby_part(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .lobby_part(raw_user_query.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn lobby_join(
        &self,
        raw_user_query: Request<RawUserQuery>,
    ) -> Result<Response<HandleCompleted>, Status> {
        let res = self
            .bancho_service
            .lobby_join(raw_user_query.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }
}
