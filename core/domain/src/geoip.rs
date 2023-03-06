use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GeoipData {
    pub location: Location,
    pub continent: Continent,
    pub country: Country,
    pub region: Region,
    pub city: City,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Location {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Continent {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Country {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Region {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct City {
    pub geoname_id: Option<u32>,
    pub name: Option<String>,
}
