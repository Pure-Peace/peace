#![allow(unused_variables)]
#![allow(unused_imports)]

use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};

use actix_multipart::{Field, Multipart};
use actix_web::web::Bytes;
use actix_web::HttpRequest;

use argon2::verify_encoded;
use futures::StreamExt;
use maxminddb::{geoip2::City, Reader};
use memmap::Mmap;
use serde_qs;

use crate::constants::{GeoData, GeoError};

#[inline(always)]
/// Get deserialized multipart/form-data
pub async fn get_form_data<T: serde::de::DeserializeOwned>(
    mut payload: Multipart,
) -> Result<T, serde_qs::Error> {
    let mut query: String = String::new();

    while let Some(item) = payload.next().await {
        let mut field: Field = item.unwrap();
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();
        while let Some(chunk) = field.next().await {
            let value = String::from_utf8(chunk.unwrap().to_vec()).unwrap();
            query.push_str(&format!("{}={}&", name, value));
        }
    }
    serde_qs::from_str(&query)
}

#[inline(always)]
/// Get real ip from request
pub async fn get_realip(req: &HttpRequest) -> Result<String, ()> {
    match req.connection_info().realip_remote_addr() {
        Some(ip) => Ok(ip.to_string()),
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
    verify_encoded(password_crypted, password.as_bytes()).unwrap_or_else(|err| {
        error!(
            "failed to verify argon2: {:?}; crypted: {}, password: {}",
            err, password_crypted, password
        );
        false
    })
}

#[inline(always)]
/// Query geo-ip info from local database
pub async fn geo_ip_info(
    ip_address: &String,
    geo_db: &Option<Reader<Mmap>>,
    query: &HashMap<String, String>,
) -> Result<String, String> {
    match geo_db {
        Some(reader) => {
            if !ip_address.parse::<Ipv4Addr>().is_ok() && !ip_address.parse::<Ipv6Addr>().is_ok() {
                return Err(GeoError::new(ip_address, Some("wrong ip address")));
            };

            match reader.lookup::<City>(ip_address.parse().unwrap()) {
                Ok(geo_data) => {
                    let json_data = {
                        let lang: &str = match query.get("lang") {
                            Some(s) => s.as_ref(),
                            None => "en",
                        };

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
                            .cloned();

                        let country_name = geo_data
                            .country
                            .as_ref()
                            .and_then(|country| country.names.as_ref())
                            .and_then(|names| names.get(lang))
                            .cloned();

                        let region_name = region
                            .and_then(|region| region.names.as_ref())
                            .and_then(|names| names.get(lang))
                            .cloned();

                        let city_name = geo_data
                            .city
                            .as_ref()
                            .and_then(|c| c.names.as_ref())
                            .and_then(|b| b.get(&lang))
                            .cloned();

                        let latitude = geo_data.location.as_ref().and_then(|lo| lo.latitude);
                        let longitude = geo_data.location.as_ref().and_then(|lo| lo.longitude);

                        let continent_code = geo_data.continent.and_then(|co| co.code);
                        let country_code = geo_data.country.and_then(|cou| cou.iso_code);
                        let region_code = region.and_then(|s| s.iso_code);
                        let timezone = geo_data.location.and_then(|lo| lo.time_zone);

                        serde_json::to_string(&GeoData {
                            ip_address,
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
                        })
                    };

                    match json_data {
                        Ok(json) => Ok(json),
                        Err(err) => {
                            warn!("Failed to parse ip address: {}", ip_address);
                            return Err(GeoError::new(ip_address, Some("can not parse your ip address")));
                        }
                    }
                }

                Err(err) => {
                    warn!("Failed to get ip location info: {}; err: {:?}", ip_address, err);
                    return Err(GeoError::new(ip_address, Some("can not get your ip location info")));
                }
            }
        }
        None => Err(GeoError::new(ip_address, Some("Geo-ip service is not enabled"))),
    }
}
