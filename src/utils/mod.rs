use std::net::{Ipv4Addr, Ipv6Addr};

use actix_multipart::Multipart;
use actix_web::HttpRequest;
use argon2::{ThreadMode, Variant, Version};

use futures::StreamExt;
use lazy_static::lazy_static;
use maxminddb::{geoip2::City, Reader};
use memmap::Mmap;
use rand::Rng;
use serde_qs;

use crate::constants::GeoData;

lazy_static! {
    static ref ARGON2_CONFIG: argon2::Config<'static> = argon2::Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 4096,
        time_cost: 3,
        lanes: 1,
        thread_mode: ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32
    };
}

#[inline(always)]
/// Get deserialized multipart/form-data
pub async fn get_form_data<T: serde::de::DeserializeOwned>(
    mut form_data: Multipart,
) -> Result<T, serde_qs::Error> {
    let mut temp: String = String::new();
    while let Some(item) = form_data.next().await {
        let mut field = item.unwrap();
        if let Some(content_type) = field.content_disposition() {
            let key = content_type.get_name();
            if key.is_none() {
                continue;
            }
            while let Some(chunk) = field.next().await {
                if chunk.is_err() {
                    continue;
                }
                let value = String::from_utf8(chunk.unwrap().to_vec()).unwrap_or(String::new());
                if temp.len() > 0 {
                    temp.push('&');
                }
                temp.push_str(&format!("{}={}", key.unwrap(), value));
            }
        }
    }
    serde_qs::from_str(&temp)
}

#[inline(always)]
/// Get real ip from request
pub async fn get_realip(req: &HttpRequest) -> Result<String, ()> {
    match req.connection_info().realip_remote_addr() {
        Some(ip) => Ok(match ip.find(':') {
            Some(idx) => ip[0..idx].to_string(),
            None => ip.to_string(),
        }),
        None => Err(()),
    }
}

#[inline(always)]
/// Get osu version from headers
pub async fn get_osuver(req: &HttpRequest) -> String {
    match req.headers().get("osu-version") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}

#[inline(always)]
/// Get osu token from headers
pub async fn get_token(req: &HttpRequest) -> String {
    match req.headers().get("osu-token") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}

#[inline(always)]
/// Argon2 verify
pub async fn argon2_verify(password_crypted: &str, password: &str) -> bool {
    argon2::verify_encoded(password_crypted, password.as_bytes()).unwrap_or_else(|err| {
        error!(
            "failed to verify argon2: {:?}; crypted: {}, password: {}",
            err, password_crypted, password
        );
        false
    })
}

#[inline(always)]
/// Argon2 encode
pub async fn argon2_encode(password: &[u8]) -> String {
    argon2::hash_encoded(password, rand_string().await.as_bytes(), &ARGON2_CONFIG).unwrap()
}

#[inline(always)]
/// Argon2 encode
pub async fn rand_string() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}

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
                "Failed to get ip location info: {}; err: {:?}",
                ip_address, err
            );
            return Err("Ip address not found.".to_string());
        }
    }
}
