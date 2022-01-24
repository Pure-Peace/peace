use std::{net::IpAddr, path::Path};

use serde::Serialize;

pub use maxminddb::{geoip2, MaxMindDBError, Reader};
pub use memmap2::Mmap;

const LANG: &str = "en";

#[derive(Serialize, Debug, Clone, Default)]
pub struct GeoipData {
    pub ip_address: String,
    pub location: Location,
    pub continent: Continent,
    pub country: Country,
    pub region: Region,
    pub city: City,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Location {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Continent {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Country {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Region {
    pub geoname_id: Option<u32>,
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct City {
    pub geoname_id: Option<u32>,
    pub name: Option<String>,
}

pub trait FromMaxmindDB {
    fn city(&self, address: IpAddr) -> Result<GeoipData, MaxMindDBError>;
}

macro_rules! map_to_string {
    ($s: expr) => {
        $s.map(|s| s.to_string())
    };
}

macro_rules! get_name {
    ($i: expr) => {
        $i.names.as_ref().and_then(|n| map_to_string!(n.get(LANG)))
    };
}

impl FromMaxmindDB for Reader<Mmap> {
    fn city(&self, address: IpAddr) -> Result<GeoipData, MaxMindDBError> {
        let geo_data = self.lookup::<geoip2::City>(address)?;

        let location = geo_data
            .location
            .map(|lo| Location {
                latitude: lo.latitude,
                longitude: lo.longitude,
                timezone: map_to_string!(lo.time_zone),
            })
            .unwrap_or_default();

        let continent = geo_data
            .continent
            .map(|co| Continent {
                geoname_id: co.geoname_id,
                code: map_to_string!(co.code),
                name: get_name!(co),
            })
            .unwrap_or_default();

        let country = geo_data
            .country
            .map(|c| Country {
                geoname_id: c.geoname_id,
                code: map_to_string!(c.iso_code),
                name: get_name!(c),
            })
            .unwrap_or_default();

        let region = geo_data
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

        let city = geo_data
            .city
            .map(|c| City {
                geoname_id: c.geoname_id,
                name: get_name!(c),
            })
            .unwrap_or_default();

        Ok(GeoipData {
            ip_address: address.to_string(),
            location,
            continent,
            country,
            region,
            city,
        })
    }
}

pub fn create_mmdb<P: AsRef<Path>>(database: P) -> Result<Reader<Mmap>, MaxMindDBError> {
    Reader::open_mmap(database)
}
