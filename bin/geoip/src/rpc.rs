use peace_pb::{base::ExecSuccess, geoip_rpc::*};
use peace_services::geoip::DynGeoipService;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct GeoipRpcImpl {
    pub geoip_service: DynGeoipService,
}

impl GeoipRpcImpl {
    pub fn new(geoip_service: DynGeoipService) -> Self {
        Self { geoip_service }
    }
}

#[tonic::async_trait]
impl geoip_rpc_server::GeoipRpc for GeoipRpcImpl {
    async fn lookup_with_ip_address(
        &self,
        request: Request<IpAddress>,
    ) -> Result<Response<GeoipData>, Status> {
        /* self.bancho_state_service
        .broadcast_bancho_packets(request.into_inner())
        .await
        .map_err(|err| err.into())
        .map(|resp| Response::new(resp)) */
        unimplemented!()
    }

    async fn try_reload(
        &self,
        request: Request<GeoDbPath>,
    ) -> Result<Response<ExecSuccess>, Status> {
        unimplemented!()
    }
}
