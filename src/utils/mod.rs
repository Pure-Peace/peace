#![allow(unused_macros)]
use bytes::Bytes;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Instant,
};

use actix_multipart::Multipart;
use actix_web::{web::Data, HttpRequest};
use argon2::{ThreadMode, Variant, Version};
use hashbrown::HashMap;

use std::fmt::Display;
use std::str::FromStr;

use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use futures::StreamExt;
use lazy_static::lazy_static;
use maxminddb::{geoip2::City, Reader};
use memmap::Mmap;
use rand::Rng;
use serde::de::{self, Deserialize, Deserializer};
use serde_qs;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{constants::GeoData, database::Database, objects::PlayerBase, types::Argon2Cache};

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

#[macro_export]
macro_rules! set_with_db {
    (
        table=$table:expr;
        schema=$schema:expr;
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*$(,)+
        }
    ) => {
        $(#[$meta])*
        $vis struct $struct_name{
            $(
                $(#[$field_meta:meta])*
                $field_vis $field_name: $field_type,
            )*
        }
        paste::paste! {
            impl $struct_name {
                $(
                    pub async fn [<set_ $field_name _with_db>](&mut self, value: $field_type, database: &Database) -> bool {
                        let query = concat!(r#"UPDATE "#, stringify!($table), r#"."#, stringify!($schema), r#" SET ""#, stringify!($field_name), r#"" = $1 WHERE "id" = $2"#);
                        println!("{}", query);
                        let res = match database.pg.execute(query, &[&value, &self.id]).await {
                            Ok(_) => true,
                            Err(err) => {
                                warn!(
                                    stringify!("Failed to set "$struct_name"."$field_name" to table "$table", err: ""{:?}"),
                                    err
                                );
                                false
                            }
                        };
                        self.$field_name = value;
                        res
                    }
                )*
            }
        }
    }
}

#[inline(always)]
pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[inline(always)]
pub fn from_str_optional<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer);
    if s.is_err() {
        return Ok(None);
    };
    Ok(match T::from_str(&s.unwrap()) {
        Ok(t) => Some(t),
        Err(_) => None,
    })
}

#[inline(always)]
pub fn from_str_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let de = String::deserialize(deserializer);
    if de.is_err() {
        return Ok(false);
    };
    match de.unwrap().as_str() {
        "1" => Ok(true),
        _ => Ok(false),
    }
}

#[inline(always)]
pub fn noew_time_local() -> DateTime<Local> {
    Local::now()
}

#[inline(always)]
/// Get deserialized multipart/form-data
///
/// use query method, some data types not support (such as bytes)
pub async fn simple_get_form_data<T: serde::de::DeserializeOwned>(
    mut form_data: Multipart,
) -> Result<T, serde_qs::Error> {
    let mut temp: String = String::new();
    while let Some(item) = form_data.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_e) => continue,
        };
        if let Some(disposition) = field.content_disposition() {
            if let Some(key) = disposition.get_name() {
                while let Some(Ok(chunk)) = field.next().await {
                    let value = String::from_utf8(chunk.to_vec()).unwrap_or(String::new());
                    if temp.len() > 0 {
                        temp.push('&');
                    }
                    temp.push_str(&format!("{}={}", key, value));
                }
            }
        }
    }
    serde_qs::from_str(&temp)
}

#[inline(always)]
pub fn try_parse<T>(string: &str) -> Option<T>
where
    T: FromStr,
{
    match T::from_str(string) {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

#[derive(Debug)]
pub struct MultipartData {
    pub forms: HashMap<String, String>,
    pub files: HashMap<String, Bytes>,
}

impl MultipartData {
    #[inline(always)]
    pub fn form<T>(&mut self, key: &str) -> Option<T>
    where
        T: FromStr,
    {
        let s = self.forms.remove(key)?;
        match T::from_str(s.as_ref()) {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    #[inline(always)]
    pub fn file(&mut self, key: &str) -> Option<Bytes> {
        self.files.remove(key)
    }
}

#[inline(always)]
/// Get deserialized multipart/form-data or files
pub async fn get_mutipart_data(mut mutipart_data: Multipart) -> MultipartData {
    let mut files = HashMap::new();
    let mut forms = HashMap::new();
    while let Some(item) = mutipart_data.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_e) => continue,
        };
        if let Some(disposition) = field.content_disposition() {
            let file_name = disposition.get_filename();
            if let Some(key) = disposition.get_name() {
                while let Some(Ok(chunk)) = field.next().await {
                    if file_name.is_some() {
                        files.insert(key.to_string(), chunk);
                    } else {
                        forms.insert(
                            key.to_string(),
                            String::from_utf8(chunk.to_vec()).unwrap_or(String::new()),
                        );
                    }
                }
            }
        }
    }
    MultipartData { forms, files }
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
    let argon2_verify_start = Instant::now();
    let verify_result = argon2::verify_encoded(password_crypted, password.as_bytes());
    let argon2_verify_end = argon2_verify_start.elapsed();
    debug!("Argon2 verify done, time spent: {:.2?};", argon2_verify_end);
    match verify_result {
        Ok(result) => result,
        Err(err) => {
            error!(
                "Failed to verify argon2: {:?}; crypted: {}, password: {}",
                err, password_crypted, password
            );
            false
        }
    }
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

#[inline(always)]
pub async fn get_player_base(username: &String, database: &Database) -> Option<PlayerBase> {
    let username_safe = username.to_lowercase().replace(" ", "_");

    // Select from database
    let from_base_row = match database
        .pg
        .query_first(
            r#"SELECT 
                    "id", "name", "privileges", "country", "password"
                    FROM "user"."base" WHERE 
                    "name_safe" = $1;"#,
            &[&username_safe],
        )
        .await
    {
        Ok(from_base_row) => from_base_row,
        Err(_err) => {
            warn!(
                "failed to get playerbase: username ({}) is not exists! ",
                username
            );
            return None;
        }
    };
    // Try deserialize player base object
    match serde_postgres::from_row::<PlayerBase>(&from_base_row) {
        Ok(player_base) => Some(player_base),
        Err(err) => {
            error!(
                "could not deserialize playerbase: {}; err: {:?}",
                username, err
            );
            None
        }
    }
}

#[inline(always)]
pub async fn checking_password(
    player_base: &PlayerBase,
    password_hash: &String,
    argon2_cache: &RwLock<Argon2Cache>,
) -> bool {
    // Try read password hash from argon2 cache
    let cached_password_hash = {
        argon2_cache
            .read()
            .await
            .get(&player_base.password)
            .cloned()
    };

    // Cache hitted, checking
    if let Some(cached_password_hash) = cached_password_hash {
        debug!(
            "password cache hitted: {}({})",
            player_base.name, player_base.id
        );
        return &cached_password_hash == password_hash;
    }

    // Cache not hitted, do argon2 verify (ms level)
    let verify_result = argon2_verify(&player_base.password, password_hash).await;
    if verify_result {
        // If password is correct, cache it
        // key = argon2 cipher, value = password hash
        argon2_cache
            .write()
            .await
            .insert(player_base.password.clone(), password_hash.clone());
    }

    verify_result
}

/// Get beatmap ratings from database
#[inline(always)]
pub async fn get_beatmap_rating(beatmap_md5: &String, database: &Database) -> Option<f32> {
    match database
        .pg
        .query_first(
            r#"SELECT AVG("rating")::float4 FROM "beatmaps"."ratings" WHERE "map_md5" = $1"#,
            &[beatmap_md5],
        )
        .await
    {
        Ok(value) => Some(value.get(0)),
        Err(err) => {
            error!(
                "failed to get avg rating from beatmap {}, err: {:?}",
                beatmap_md5, err
            );
            None
        }
    }
}

#[inline(always)]
fn get_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

#[inline(always)]
/// Utils for struct from database
pub async fn struct_from_database<T: FromTokioPostgresRow>(
    table: &str,
    schema: &str,
    query_by: &str,
    fields: &str,
    param: &(dyn tokio_postgres::types::ToSql + Sync),
    database: &Database,
) -> Option<T> {
    let type_name = std::any::type_name::<T>();
    let query = format!(
        "SELECT {} FROM \"{}\".\"{}\" WHERE \"{}\" = $1;",
        fields, table, schema, query_by
    );
    debug!("struct_from_database query: {}", query);
    let row = database.pg.query_first(&query, &[param]).await;
    if let Err(err) = row {
        debug!(
            "Failed to get {} {:?} from database table {}.{} error: {:?}",
            type_name, param, table, schema, err
        );
        return None;
    }

    let row = row.unwrap();
    match <T>::from_row(row) {
        Ok(result) => Some(result),
        Err(err) => {
            error!(
                "Failed to deserialize {} from pg-row! error: {:?}",
                type_name, err
            );
            None
        }
    }
}

#[inline(always)]
pub fn lock_wrapper<T>(obj: T) -> Data<RwLock<T>> {
    Data::new(RwLock::new(obj))
}

#[inline(always)]
pub fn build_s(len: usize) -> String {
    let mut s = String::new();
    for i in 1..len + 1 {
        s += (if i == len {
            format!("${}", i)
        } else {
            format!("${},", i)
        })
        .as_str();
    }
    s
}

#[inline(always)]
pub fn safe_file_name(mut s: String) -> String {
    for i in r#":\*></?"|"#.chars() {
        s = s.replace(i, "");
    }
    s
}

#[inline(always)]
pub fn osu_sumit_token_checker(req: &HttpRequest) -> bool {
    if let Some(token) = req.headers().get("Token") {
        if let Ok(token) = token.to_str() {
            let token = token.split("|").collect::<Vec<&str>>();
            if token.len() == 2 && token[0].len() > 4000 && token[1].len() == 32 {
                return true;
            };
            warn!(
                "[osu_sumit_token_checker] Token len: {}, len[0]: {}, len[1]: {}",
                token.len(),
                token[0].len(),
                token[1].len()
            );
        };
    };
    false
}
