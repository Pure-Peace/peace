use peace_pb::geoip::{
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

impl Into<RpcGeoipData> for GeoipData {
    fn into(self) -> RpcGeoipData {
        RpcGeoipData {
            location: Some(self.location.into()),
            continent: Some(self.continent.into()),
            country: Some(self.country.into()),
            region: Some(self.region.into()),
            city: Some(self.city.into()),
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

impl Into<RpcLocation> for Location {
    fn into(self) -> RpcLocation {
        RpcLocation {
            latitude: self.latitude.into(),
            longitude: self.longitude.into(),
            timezone: self.timezone.into(),
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

impl Into<RpcContinent> for Continent {
    fn into(self) -> RpcContinent {
        RpcContinent {
            geoname_id: self.geoname_id.into(),
            code: self.code.into(),
            name: self.name.into(),
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

impl Into<RpcCountry> for Country {
    fn into(self) -> RpcCountry {
        RpcCountry {
            geoname_id: self.geoname_id.into(),
            code: self.code.into(),
            name: self.name.into(),
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

impl Into<RpcRegion> for Region {
    fn into(self) -> RpcRegion {
        RpcRegion {
            geoname_id: self.geoname_id.into(),
            code: self.code.into(),
            name: self.name.into(),
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

impl Into<RpcCity> for City {
    fn into(self) -> RpcCity {
        RpcCity { geoname_id: self.geoname_id.into(), name: self.name.into() }
    }
}
