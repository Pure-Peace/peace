use super::GeoipError;
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use maxminddb::Reader;
use memmap2::Mmap;
use peace_domain::geoip::GeoipData;
use peace_pb::base::ExecSuccess;
use std::{net::IpAddr, sync::Arc};

pub type DynGeoipService = Arc<dyn GeoipService + Send + Sync>;
pub type GeoDb = Arc<Reader<Mmap>>;
pub type ReloadableGeoDb = Arc<ArcSwapOption<Reader<Mmap>>>;

pub trait GeoipService: LookupIpAddress + ReloadGeoDb {}

#[async_trait]
pub trait LookupIpAddress {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError>;
}

#[async_trait]
pub trait ReloadGeoDb {
    async fn try_reload(&self, path: &str) -> Result<ExecSuccess, GeoipError>;
}

pub trait FromGeoDb {
    fn from_geo_db(db: GeoDb) -> Self;
}

pub trait FromGeoDbPath {
    fn from_path(path: &str) -> Result<Self, GeoipError>
    where
        Self: Sized + FromGeoDb,
    {
        Ok(Self::from_geo_db(super::load_db(path)?))
    }
}
