use super::{DynGeoipService, GeoipError, GeoipService};
use crate::rpc_config::GeoipRpcConfig;
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use maxminddb::{geoip2, Mmap, Reader};
use peace_api::RpcClientConfig;
use peace_domain::geoip::*;
use peace_pb::geoip::{geoip_rpc_client::GeoipRpcClient, GeoDbPath, IpAddress};
use std::{net::IpAddr, sync::Arc};
use tonic::transport::Channel;

const LANGUAGE: &str = "en";
const DEFAULT_GEO_DB_PATH: &str = "GeoLite2-City.mmdb";

pub struct GeoipServiceBuilder;

impl GeoipServiceBuilder {
    pub async fn build(
        geo_db_path: Option<&str>,
        geoip_rpc_config: Option<&GeoipRpcConfig>,
    ) -> DynGeoipService {
        info!("initializing Geoip service...");
        let mut service = GeoipServiceImpl::from_path(
            geo_db_path.unwrap_or(DEFAULT_GEO_DB_PATH),
        )
        .ok()
        .map(|svc| svc.into_service());

        if service.is_some() {
            info!("Geoip service init successful, type: \"Local\"");
            return service.unwrap()
        }

        if let Some(cfg) = geoip_rpc_config {
            service = cfg
                .connect_client()
                .await
                .map(|client| {
                    info!("Geoip service init successful, type: \"Remote\"");
                    GeoipServiceRemote::new(client).into_service()
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
            GeoipServiceImpl::lazy_init().into_service()
        })
    }
}

#[derive(Debug, Clone)]
pub struct GeoipServiceRemote(GeoipRpcClient<Channel>);

impl GeoipServiceRemote {
    pub fn new(geoip_rpc_client: GeoipRpcClient<Channel>) -> Self {
        Self(geoip_rpc_client)
    }

    pub fn into_service(self) -> DynGeoipService {
        Arc::new(self) as DynGeoipService
    }

    pub fn client(&self) -> GeoipRpcClient<Channel> {
        self.0.clone()
    }
}

#[derive(Clone, Default)]
pub struct GeoipServiceImpl {
    geo_db: Arc<ArcSwapOption<Reader<Mmap>>>,
}

impl GeoipServiceImpl {
    pub fn new(geo_db: Arc<Reader<Mmap>>) -> Self {
        Self { geo_db: Arc::new(ArcSwapOption::new(Some(geo_db))) }
    }

    pub fn into_service(self) -> DynGeoipService {
        Arc::new(self) as DynGeoipService
    }

    pub fn from_path(path: &str) -> Result<Self, GeoipError> {
        Ok(Self::new(GeoipServiceImpl::load_db(path)?))
    }

    pub fn lazy_init() -> Self {
        Self::default()
    }

    pub fn load_db(geo_db_path: &str) -> Result<Arc<Reader<Mmap>>, GeoipError> {
        Reader::open_mmap(geo_db_path)
            .map(Arc::new)
            .map_err(GeoipError::FailedToLoadDatabase)
    }
}

#[async_trait]
impl GeoipService for GeoipServiceImpl {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError> {
        let geo_db =
            self.geo_db.load_full().ok_or(GeoipError::NotInitialized)?;
        let data = geo_db
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

    async fn try_reload(&self, geo_db_path: &str) -> Result<(), GeoipError> {
        self.geo_db.store(Some(GeoipServiceImpl::load_db(geo_db_path)?));
        Ok(())
    }
}

#[async_trait]
impl GeoipService for GeoipServiceRemote {
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

    async fn try_reload(&self, geo_db_path: &str) -> Result<(), GeoipError> {
        self.client()
            .try_reload(GeoDbPath { geo_db_path: geo_db_path.to_owned() })
            .await
            .map_err(GeoipError::RpcError)
            .map(|_| ())
    }
}
