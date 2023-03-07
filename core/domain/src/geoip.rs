use peace_pb::geoip_rpc::{
    City as RpcCity, Continent as RpcContinent, Country as RpcCountry,
    GeoipData as RpcGeoipData, Location as RpcLocation, Region as RpcRegion,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GeoipData {
    pub location: Location,
    pub continent: Continent,
    pub country: Country,
    pub region: Region,
    pub city: City,
}

impl From<RpcGeoipData> for GeoipData {
    fn from(resp: RpcGeoipData) -> Self {
        Self {
            location: resp.location.unwrap().into(),
            continent: resp.continent.unwrap().into(),
            country: resp.country.unwrap().into(),
            region: resp.region.unwrap().into(),
            city: resp.city.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Location {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl From<RpcLocation> for Location {
    fn from(resp: RpcLocation) -> Self {
        Self {
            latitude: resp.latitude.unwrap().into(),
            longitude: resp.longitude.unwrap().into(),
            timezone: resp.timezone.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Continent {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

impl From<RpcContinent> for Continent {
    fn from(resp: RpcContinent) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap().into(),
            code: resp.code.unwrap().into(),
            name: resp.name.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Country {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

impl From<RpcCountry> for Country {
    fn from(resp: RpcCountry) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap().into(),
            code: resp.code.unwrap().into(),
            name: resp.name.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Region {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

impl From<RpcRegion> for Region {
    fn from(resp: RpcRegion) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap().into(),
            code: resp.code.unwrap().into(),
            name: resp.name.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct City {
    pub geoname_id: Option<u32>,
    pub name: Option<String>,
}

impl From<RpcCity> for City {
    fn from(resp: RpcCity) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap().into(),
            name: resp.name.unwrap().into(),
        }
    }
}
