use std::net::IpAddr;

use peace_pb::{
    base::ExecSuccess,
    geoip_rpc::{GeoipData as RpcGeoipData, *},
};
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
    ) -> Result<Response<RpcGeoipData>, Status> {
        self.geoip_service
            .lookup_with_ip_address(
                request
                    .into_inner()
                    .ip
                    .parse::<IpAddr>()
                    .map_err(|err| Status::invalid_argument(err.to_string()))?,
            )
            .await
            .map_err(|err| err.into())
            .map(|data| Response::new(data.into()))
    }

    async fn try_reload(
        &self,
        request: Request<GeoDbPath>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.geoip_service
            .try_reload(&request.into_inner().geo_db_path)
            .await
            .map_err(|err| err.into())
            .map(|()| Response::new(ExecSuccess {}))
    }
}
