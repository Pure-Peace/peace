use maxminddb::{geoip2::City, Reader};
use memmap::Mmap;
use peace_constants::GeoData;
use std::net::{Ipv4Addr, Ipv6Addr};

#[inline(always)]
/// Query geo-ip info from local database
pub async fn geo_ip_info(
    ip_address: &String,
    geo_db: &Option<Reader<Mmap>>,
) -> Result<String, String> {
    if geo_db.is_none() {
        return Err("Geo-ip service is not enabled.".to_string());
    }

    let geo_data = get_geo_ip_data(ip_address, &geo_db.as_ref().unwrap())?;

    let json_data = serde_json::to_string(&geo_data);
    if json_data.is_err() {
        return Err("Failed to parse data.".to_string());
    }

    Ok(json_data.unwrap())
}

#[inline(always)]
/// Query geo-ip data from local database
pub fn get_geo_ip_data(ip_address: &String, geo_db: &Reader<Mmap>) -> Result<GeoData, String> {
    if !ip_address.parse::<Ipv4Addr>().is_ok() && !ip_address.parse::<Ipv6Addr>().is_ok() {
        return Err("Bad ip address.".to_string());
    };

    match geo_db.lookup::<City>(ip_address.parse().unwrap()) {
        Ok(geo_data) => {
            let lang = "en";

            let region = geo_data
                .subdivisions
                .as_ref()
                .filter(|regions| !regions.is_empty())
                .and_then(|regions| regions.get(0));

            let continent_name = geo_data
                .continent
                .as_ref()
                .and_then(|country| country.names.as_ref())
                .and_then(|names| names.get(lang))
                .map(|s| s.to_string());

            let country_name = geo_data
                .country
                .as_ref()
                .and_then(|country| country.names.as_ref())
                .and_then(|names| names.get(lang))
                .map(|s| s.to_string());

            let region_name = region
                .and_then(|region| region.names.as_ref())
                .and_then(|names| names.get(lang))
                .map(|s| s.to_string());

            let city_name = geo_data
                .city
                .as_ref()
                .and_then(|c| c.names.as_ref())
                .and_then(|b| b.get(lang))
                .map(|s| s.to_string());

            let latitude = geo_data
                .location
                .as_ref()
                .and_then(|lo| lo.latitude)
                .unwrap_or(0.0);
            let longitude = geo_data
                .location
                .as_ref()
                .and_then(|lo| lo.longitude)
                .unwrap_or(0.0);

            let continent_code = geo_data
                .continent
                .and_then(|co| co.code)
                .map(|s| s.to_string());
            let country_code = geo_data
                .country
                .and_then(|cou| cou.iso_code)
                .map(|s| s.to_string());
            let region_code = region.and_then(|s| s.iso_code).map(|s| s.to_string());
            let timezone = geo_data
                .location
                .and_then(|lo| lo.time_zone)
                .map(|s| s.to_string());

            return Ok(GeoData {
                ip_address: ip_address.clone(),
                latitude,
                longitude,
                continent_code,
                continent_name,
                country_code,
                country_name,
                region_code,
                region_name,
                city_name,
                timezone,
                message: None,
                status_code: 1,
            });
        }

        Err(err) => {
            warn!(
                "[get_geo_ip_data] Failed to get ip location info: {}; err: {:?}",
                ip_address, err
            );
            return Err("Ip address not found.".to_string());
        }
    }
}
