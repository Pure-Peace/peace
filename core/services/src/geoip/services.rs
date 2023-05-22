use super::{
    DynGeoipService, FromClient, FromGeoDb, FromPath, GeoDb, GeoipError,
    GeoipService, GeoipServiceRpc, IntoGeoipService, LookupIpAddress,
    ReloadGeoDb, ReloadableGeoDb,
};
use crate::rpc_config::GeoipRpcConfig;
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use maxminddb::{geoip2, Reader};
use peace_api::RpcClientConfig;
use peace_domain::geoip::*;
use peace_pb::geoip::{geoip_rpc_client::GeoipRpcClient, GeoDbPath, IpAddress};
use std::{net::IpAddr, path::Path, sync::Arc};
use tonic::transport::Channel;

const LANGUAGE: &str = "en";
const DEFAULT_GEO_DB_PATH: &str = "GeoLite2-City.mmdb";

pub struct GeoipServiceBuilder;

impl GeoipServiceBuilder {
    pub async fn build<I, R>(
        path: Option<&str>,
        cfg: Option<&GeoipRpcConfig>,
    ) -> DynGeoipService
    where
        I: IntoGeoipService + FromPath + FromGeoDb + Default,
        R: IntoGeoipService + FromClient,
    {
        info!("initializing Geoip service...");
        let mut service = I::from_path(path.unwrap_or(DEFAULT_GEO_DB_PATH))
            .ok()
            .map(|svc| svc.into_service());

        if service.is_some() {
            info!("Geoip service init successful, type: \"Local\"");
            return service.unwrap()
        }

        if let Some(cfg) = cfg {
            service = cfg
                .connect_client()
                .await
                .map(|client| {
                    info!("Geoip service init successful, type: \"Remote\"");
                    R::from_client(client).into_service()
                })
                .ok();
        }

        service.unwrap_or_else(|| {
            warn!(
                "
        Geoip service init failed, will not be able to use related features!

        Please make sure you have downloaded the \"GeoLite2 City\" database
        and put it in the specified location (\"GeoLite2-City.mmdb\").
        If you have not downloaded it,
        please register and log in to your account here:
        \"https://www.maxmind.com/en/accounts/470006/geoip/downloads\"
"
            );
            I::default().into_service()
        })
    }
}

#[derive(Clone, Default)]
pub struct GeoipServiceImpl {
    pub db: ReloadableGeoDb,
}

impl GeoipServiceImpl {
    #[inline]
    pub fn new(db: GeoDb) -> Self {
        Self { db: Arc::new(ArcSwapOption::new(Some(db))) }
    }
}

#[inline]
pub fn load_db<P>(path: P) -> Result<GeoDb, GeoipError>
where
    P: AsRef<Path>,
{
    Reader::open_mmap(path)
        .map(Arc::new)
        .map_err(GeoipError::FailedToLoadDatabase)
}

impl FromPath for GeoipServiceImpl {}

impl FromGeoDb for GeoipServiceImpl {
    #[inline]
    fn from_geo_db(db: GeoDb) -> Self {
        Self { db: Arc::new(ArcSwapOption::new(Some(db))) }
    }
}

impl GeoipService for GeoipServiceImpl {}

impl IntoGeoipService for GeoipServiceImpl {}

#[async_trait]
impl LookupIpAddress for GeoipServiceImpl {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError> {
        let db = self.db.load_full().ok_or(GeoipError::NotInitialized)?;
        let data = db
            .lookup::<geoip2::City>(ip_addr)
            .map_err(GeoipError::LookupError)?;

        let location = data
            .location
            .as_ref()
            .map(|lo| Location {
                latitude: lo.latitude.unwrap_or_default(),
                longitude: lo.longitude.unwrap_or_default(),
                timezone: lo
                    .time_zone
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let continent = data
            .continent
            .as_ref()
            .map(|co| Continent {
                geoname_id: co.geoname_id.unwrap_or_default(),
                code: co.code.map(|s| s.to_string()).unwrap_or_default(),
                name: co
                    .names
                    .as_ref()
                    .and_then(|names| names.get(LANGUAGE))
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let country = data
            .country
            .as_ref()
            .map(|c| Country {
                geoname_id: c.geoname_id.unwrap_or_default(),
                code: c.iso_code.map(|s| s.to_string()).unwrap_or_default(),
                name: c
                    .names
                    .as_ref()
                    .and_then(|names| names.get(LANGUAGE))
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let region = data
            .subdivisions
            .as_ref()
            .filter(|regions| !regions.is_empty())
            .and_then(|regions| regions.get(0))
            .map(|r| Region {
                geoname_id: r.geoname_id.unwrap_or_default(),
                code: r.iso_code.map(|s| s.to_string()).unwrap_or_default(),
                name: r
                    .names
                    .as_ref()
                    .and_then(|names| names.get(LANGUAGE))
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let city = data
            .city
            .as_ref()
            .map(|c| City {
                geoname_id: c.geoname_id.unwrap_or_default(),
                name: c
                    .names
                    .as_ref()
                    .and_then(|names| names.get(LANGUAGE))
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        Ok(GeoipData { location, continent, country, region, city })
    }
}

#[async_trait]
impl ReloadGeoDb for GeoipServiceImpl {
    async fn try_reload(&self, path: &str) -> Result<(), GeoipError> {
        self.db.store(Some(load_db(path)?));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GeoipServiceRemote(GeoipRpcClient<Channel>);

impl GeoipServiceRpc for GeoipServiceRemote {
    #[inline]
    fn client(&self) -> GeoipRpcClient<Channel> {
        self.0.clone()
    }
}

impl FromClient for GeoipServiceRemote {
    fn from_client(client: GeoipRpcClient<Channel>) -> Self {
        Self(client)
    }
}

impl GeoipService for GeoipServiceRemote {}

impl IntoGeoipService for GeoipServiceRemote {}

#[async_trait]
impl LookupIpAddress for GeoipServiceRemote {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError> {
        self.client()
            .lookup_with_ip_address(IpAddress { ip: ip_addr.to_string() })
            .await
            .map_err(GeoipError::RpcError)
            .map(|resp| resp.into_inner().into())
    }
}

#[async_trait]
impl ReloadGeoDb for GeoipServiceRemote {
    async fn try_reload(&self, path: &str) -> Result<(), GeoipError> {
        self.client()
            .try_reload(GeoDbPath { geo_db_path: path.to_owned() })
            .await
            .map_err(GeoipError::RpcError)
            .map(|_| ())
    }
}
