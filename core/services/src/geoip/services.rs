use super::{DynGeoipService, GeoipError, GeoipService};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use maxminddb::{geoip2, Mmap, Reader};
use peace_domain::geoip::*;
use peace_pb::geoip_rpc::{geoip_rpc_client::GeoipRpcClient, IpAddress};
use std::{net::IpAddr, sync::Arc};
use tonic::transport::Channel;

macro_rules! map_to_string {
    ($s: expr) => {
        $s.map(|s| s.to_string())
    };
}

macro_rules! get_name {
    ($i: expr) => {
        $i.names.as_ref().and_then(|n| map_to_string!(n.get("en")))
    };
}

#[derive(Clone)]
pub enum GeoipServiceImpl {
    Remote(GeoipServiceRemote),
    Local(GeoipServiceLocal),
}

impl GeoipServiceImpl {
    pub fn into_service(self) -> DynGeoipService {
        Arc::new(self) as DynGeoipService
    }

    pub fn remote(geoip_rpc_client: GeoipRpcClient<Channel>) -> Self {
        Self::Remote(GeoipServiceRemote(geoip_rpc_client))
    }

    pub fn local(geoip_service_local: GeoipServiceLocal) -> Self {
        Self::Local(geoip_service_local)
    }
}

#[derive(Debug, Clone)]
pub struct GeoipServiceRemote(GeoipRpcClient<Channel>);

impl GeoipServiceRemote {
    pub fn new(geoip_rpc_client: GeoipRpcClient<Channel>) -> Self {
        Self(geoip_rpc_client)
    }

    pub fn client(&self) -> GeoipRpcClient<Channel> {
        self.0.clone()
    }
}

#[derive(Clone, Default)]
pub struct GeoipServiceLocal {
    geo_db: Arc<ArcSwapOption<Reader<Mmap>>>,
}

impl GeoipServiceLocal {
    pub fn new(geo_db: Arc<Reader<Mmap>>) -> Self {
        Self { geo_db: Arc::new(ArcSwapOption::new(Some(geo_db))) }
    }

    pub fn from_path(path: &str) -> Self {
        let geo_db = GeoipServiceLocal::load_db(path).expect(
            "
        Please make sure you have downloaded the `GeoLite2 City` database
        and put it in the specified location (`GeoLite2-City.mmdb`).
        If you have not downloaded it,
        please register and log in to your account here:
        `https://www.maxmind.com /en/accounts/470006/geoip/downloads`
        ",
        );
        Self::new(geo_db)
    }

    pub fn lazy_init() -> Self {
        Self::default()
    }

    pub fn load_db(geo_db_path: &str) -> Result<Arc<Reader<Mmap>>, GeoipError> {
        Reader::open_mmap(geo_db_path)
            .map(|db| Arc::new(db))
            .map_err(GeoipError::FailedToLoadDatabase)
    }
}

#[async_trait]
impl GeoipService for GeoipServiceImpl {
    async fn lookup_with_ip_address(
        &self,
        ip_addr: IpAddr,
    ) -> Result<GeoipData, GeoipError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .lookup_with_ip_address(IpAddress { ip: ip_addr.to_string() })
                .await
                .map_err(GeoipError::RpcError)
                .map(|resp| resp.into_inner().into()),
            Self::Local(svc) => {
                let geo_db =
                    svc.geo_db.load_full().ok_or(GeoipError::NotInitialized)?;
                let data = geo_db
                    .lookup::<geoip2::City>(ip_addr)
                    .map_err(GeoipError::LookupError)?;

                let location = data
                    .location
                    .as_ref()
                    .map(|lo| Location {
                        latitude: lo.latitude,
                        longitude: lo.longitude,
                        timezone: map_to_string!(lo.time_zone),
                    })
                    .unwrap_or_default();

                let continent = data
                    .continent
                    .as_ref()
                    .map(|co| Continent {
                        geoname_id: co.geoname_id,
                        code: map_to_string!(co.code),
                        name: get_name!(co),
                    })
                    .unwrap_or_default();

                let country = data
                    .country
                    .as_ref()
                    .map(|c| Country {
                        geoname_id: c.geoname_id,
                        code: map_to_string!(c.iso_code),
                        name: get_name!(c),
                    })
                    .unwrap_or_default();

                let region = data
                    .subdivisions
                    .as_ref()
                    .filter(|regions| !regions.is_empty())
                    .and_then(|regions| regions.get(0))
                    .map(|r| Region {
                        geoname_id: r.geoname_id,
                        code: map_to_string!(r.iso_code),
                        name: get_name!(r),
                    })
                    .unwrap_or_default();

                let city = data
                    .city
                    .as_ref()
                    .map(|c| City {
                        geoname_id: c.geoname_id,
                        name: get_name!(c),
                    })
                    .unwrap_or_default();

                Ok(GeoipData { location, continent, country, region, city })
            },
        }
    }

    async fn try_reload(&self, geo_db_path: &str) -> Result<(), GeoipError> {
        match self {
            Self::Remote(_) => Err(GeoipError::OnlyLocalService),
            Self::Local(svc) => {
                svc.geo_db
                    .store(Some(GeoipServiceLocal::load_db(geo_db_path)?));
                Ok(())
            },
        }
    }
}
