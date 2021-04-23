use actix_multipart::Multipart;
use bytes::Bytes;
use futures::StreamExt;
use hashbrown::HashMap;
use std::str::FromStr;

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

#[cfg(feature = "actix_web")]
use actix_web::{middleware::Logger, web::Data, HttpRequest};
#[cfg(feature = "actix_web")]
use async_std::sync::RwLock;

#[cfg(feature = "actix_web")]
#[inline(always)]
pub fn lock_wrapper<T>(obj: T) -> Data<RwLock<T>> {
    Data::new(RwLock::new(obj))
}

#[cfg(feature = "actix_web")]
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

#[cfg(feature = "actix_web")]
#[inline(always)]
/// Get osu version from headers
pub async fn get_osuver(req: &HttpRequest) -> String {
    match req.headers().get("osu-version") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}

#[cfg(feature = "actix_web")]
#[inline(always)]
/// Get osu token from headers
pub async fn get_token(req: &HttpRequest) -> String {
    match req.headers().get("osu-token") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    }
}

#[cfg(feature = "actix_web")]
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

#[cfg(feature = "actix_web")]
pub fn make_logger(
    log_format: &str,
    exclude_target_endpoint: bool,
    target_endpoint: &str,
    exclude_endpoints: &Vec<String>,
    exclude_endpoints_regex: &Vec<String>,
) -> Logger {
    let mut logger = match exclude_target_endpoint {
        true => Logger::new(log_format).exclude(target_endpoint),
        false => Logger::new(log_format),
    };
    for i in exclude_endpoints.iter() {
        logger = logger.exclude(i as &str);
    }
    for i in exclude_endpoints_regex.iter() {
        logger = logger.exclude_regex(i as &str);
    }
    logger
}
