use actix_multipart::{Field, Multipart};
use actix_web::web::Bytes;

use futures::StreamExt;

use std::collections::HashMap;

use serde_qs;


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
