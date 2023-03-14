use crate::geoip::{City, Continent, Country, Location, Region};
use peace_pb::{
    bancho_state::ConnectionInfo as RpcConnectionInfo,
    geoip::GeoipData as RpcGeoipData,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct CreateSessionDto {
    pub user_id: i32,
    pub username: String,
    pub username_unicode: Option<String>,
    pub privileges: i32,
    pub client_version: String,
    pub utc_offset: u8,
    pub display_city: bool,
    pub only_friend_pm_allowed: bool,
    pub connection_info: ConnectionInfo,
    pub initial_packets: Option<Vec<u8>>,
}

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

impl Into<RpcConnectionInfo> for ConnectionInfo {
    fn into(self) -> RpcConnectionInfo {
        RpcConnectionInfo {
            ip: self.ip,
            geoip_data: Some(RpcGeoipData {
                location: Some(self.location.into()),
                continent: Some(self.continent.into()),
                country: Some(self.country.into()),
                region: Some(self.region.into()),
                city: Some(self.city.into()),
            }),
        }
    }
}
