use geoip_service::DynGeoipService;
use peace_pb::{
    base::ExecSuccess,
    geoip::{GeoipData as RpcGeoipData, *},
};
use std::net::IpAddr;
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
        let ip_addr = request
            .into_inner()
            .ip
            .parse::<IpAddr>()
            .map_err(|err| Status::internal(err.to_string()))?;

        let res = self.geoip_service.lookup_with_ip_address(ip_addr).await?;

        Ok(Response::new(res.into()))
    }

    async fn try_reload(
        &self,
        request: Request<GeoDbPath>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .geoip_service
            .try_reload(&request.into_inner().geo_db_path)
            .await?;

        Ok(Response::new(res))
    }
}
