use super::GeoipError;
use async_trait::async_trait;
use peace_domain::geoip::GeoipData;
use std::{net::IpAddr, sync::Arc};

pub type DynGeoipService = Arc<dyn GeoipService + Send + Sync>;

#[async_trait]
pub trait GeoipService {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError>;

    async fn try_reload(&self, geo_db_path: &str) -> Result<(), GeoipError>;
}
