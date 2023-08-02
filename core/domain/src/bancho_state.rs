use crate::geoip::{City, Continent, Country, Location, Region};
use peace_pb::{
    bancho_state::ConnectionInfo as RpcConnectionInfo,
    geoip::GeoipData as RpcGeoipData,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ConnectionInfo {
    pub ip: String,
    pub location: Location,
    pub continent: Continent,
    pub country: Country,
    pub region: Region,
    pub city: City,
}

impl From<RpcConnectionInfo> for ConnectionInfo {
    fn from(info: RpcConnectionInfo) -> Self {
        let RpcGeoipData { location, continent, country, region, city } =
            info.geoip_data.unwrap_or_default();

        Self {
            ip: info.ip,
            location: location.unwrap_or_default().into(),
            continent: continent.unwrap_or_default().into(),
            country: country.unwrap_or_default().into(),
            region: region.unwrap_or_default().into(),
            city: city.unwrap_or_default().into(),
        }
    }
}

impl From<ConnectionInfo> for RpcConnectionInfo {
    fn from(val: ConnectionInfo) -> Self {
        RpcConnectionInfo {
            ip: val.ip,
            geoip_data: Some(RpcGeoipData {
                location: Some(val.location.into()),
                continent: Some(val.continent.into()),
                country: Some(val.country.into()),
                region: Some(val.region.into()),
                city: Some(val.city.into()),
            }),
        }
    }
}
