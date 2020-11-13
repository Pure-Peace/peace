#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_multipart::{Field, Multipart};
use actix_web::HttpRequest;
use actix_web::web::Bytes;

use futures::StreamExt;

use std::collections::HashMap;

use serde_qs;


/// Get deserialized multipart/form-data
pub async fn get_form_data<T: serde::de::DeserializeOwned> (payload: &mut Multipart) -> T {
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
    serde_qs::from_str(&query).unwrap()
}


/// Get real ip from request
pub async fn get_realip (req: &HttpRequest) -> Result<String, ()> {
    match req.connection_info().realip_remote_addr() {
        Some(ip) => Ok(ip.to_string()),
        None => Err(()),
    }
}

/// Get osu version from headers
pub async fn get_osuver (req: &HttpRequest) -> String {
    match req.headers().get("osu-version") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}

/// Get osu token from headers
pub async fn get_token (req: &HttpRequest) -> String {
    match req.headers().get("osu-token") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}