use super::GeoipError;
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use maxminddb::Reader;
use memmap2::Mmap;
use peace_domain::geoip::GeoipData;
use peace_pb::geoip::geoip_rpc_client::GeoipRpcClient;
use std::{net::IpAddr, sync::Arc};
use tonic::transport::Channel;

pub type DynGeoipService = Arc<dyn GeoipService + Send + Sync>;
pub type GeoDb = Arc<Reader<Mmap>>;
pub type ReloadableGeoDb = Arc<ArcSwapOption<Reader<Mmap>>>;

pub trait GeoipService: LookupIpAddress + ReloadGeoDb {}

pub trait IntoGeoipService:
    GeoipService + Sized + Sync + Send + 'static
{
    fn into_service(self) -> DynGeoipService {
        Arc::new(self) as DynGeoipService
    }
}

#[async_trait]
pub trait LookupIpAddress {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError>;
}

#[async_trait]
pub trait ReloadGeoDb {
    async fn try_reload(&self, path: &str) -> Result<(), GeoipError>;
}

pub trait FromGeoDb {
    fn from_geo_db(db: GeoDb) -> Self;
}

pub trait FromPath {
    fn from_path(path: &str) -> Result<Self, GeoipError>
    where
        Self: Sized + FromGeoDb,
    {
        Ok(Self::from_geo_db(super::load_db(path)?))
    }
}

pub trait FromClient {
    fn from_client(client: GeoipRpcClient<Channel>) -> Self;
}

pub trait GeoipServiceRpc {
    fn client(&self) -> GeoipRpcClient<Channel>;
}
