use serde::Serialize;
use serde_qs;

#[derive(Serialize, Debug)]
pub struct GeoData<'a> {
    pub ip_address: &'a str,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub continent_code: Option<&'a str>,
    pub continent_name: Option<&'a str>,
    pub country_code: Option<&'a str>,
    pub country_name: Option<&'a str>,
    pub region_code: Option<&'a str>,
    pub region_name: Option<&'a str>,
    pub city_name: Option<&'a str>,
    pub timezone: Option<&'a str>,
    pub message: Option<&'a str>,
    pub status_code: i32,
}

#[derive(Serialize, Debug)]
pub struct GeoError<'a> {
    pub ip_address: &'a str,
    pub message: Option<&'a str>,
    pub status_code: i32,
}

impl<'a> GeoError<'a> {
    #[inline(always)]
    pub fn new(ip_address: &'a str, message: Option<&'a str>) -> String {
        serde_json::to_string(&GeoError {
            ip_address,
            message,
            status_code: -1,
        })
        .unwrap()
    }
}
