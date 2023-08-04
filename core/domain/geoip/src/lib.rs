use pb_geoip::{
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
            location: resp.location.unwrap_or_default().into(),
            continent: resp.continent.unwrap_or_default().into(),
            country: resp.country.unwrap_or_default().into(),
            region: resp.region.unwrap_or_default().into(),
            city: resp.city.unwrap_or_default().into(),
        }
    }
}

impl From<GeoipData> for RpcGeoipData {
    fn from(val: GeoipData) -> Self {
        RpcGeoipData {
            location: Some(val.location.into()),
            continent: Some(val.continent.into()),
            country: Some(val.country.into()),
            region: Some(val.region.into()),
            city: Some(val.city.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

impl From<RpcLocation> for Location {
    fn from(resp: RpcLocation) -> Self {
        Self {
            latitude: resp.latitude.unwrap_or_default(),
            longitude: resp.longitude.unwrap_or_default(),
            timezone: resp.timezone.unwrap_or_default(),
        }
    }
}

impl From<Location> for RpcLocation {
    fn from(val: Location) -> Self {
        RpcLocation {
            latitude: val.latitude.into(),
            longitude: val.longitude.into(),
            timezone: val.timezone.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Continent {
    pub geoname_id: u32,
    pub code: String,
    pub name: String,
}

impl From<RpcContinent> for Continent {
    fn from(resp: RpcContinent) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap_or_default(),
            code: resp.code.unwrap_or_default(),
            name: resp.name.unwrap_or_default(),
        }
    }
}

impl From<Continent> for RpcContinent {
    fn from(val: Continent) -> Self {
        RpcContinent {
            geoname_id: val.geoname_id.into(),
            code: val.code.into(),
            name: val.name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Country {
    pub geoname_id: u32,
    pub code: String,
    pub name: String,
}

impl From<RpcCountry> for Country {
    fn from(resp: RpcCountry) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap_or_default(),
            code: resp.code.unwrap_or_default(),
            name: resp.name.unwrap_or_default(),
        }
    }
}

impl From<Country> for RpcCountry {
    fn from(val: Country) -> Self {
        RpcCountry {
            geoname_id: val.geoname_id.into(),
            code: val.code.into(),
            name: val.name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Region {
    pub geoname_id: u32,
    pub code: String,
    pub name: String,
}

impl From<RpcRegion> for Region {
    fn from(resp: RpcRegion) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap_or_default(),
            code: resp.code.unwrap_or_default(),
            name: resp.name.unwrap_or_default(),
        }
    }
}

impl From<Region> for RpcRegion {
    fn from(val: Region) -> Self {
        RpcRegion {
            geoname_id: val.geoname_id.into(),
            code: val.code.into(),
            name: val.name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct City {
    pub geoname_id: u32,
    pub name: String,
}

impl From<RpcCity> for City {
    fn from(resp: RpcCity) -> Self {
        Self {
            geoname_id: resp.geoname_id.unwrap_or_default(),
            name: resp.name.unwrap_or_default(),
        }
    }
}

impl From<City> for RpcCity {
    fn from(val: City) -> Self {
        RpcCity { geoname_id: val.geoname_id.into(), name: val.name.into() }
    }
}
