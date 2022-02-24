use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct GeoData {
    pub ip_address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub continent_code: Option<String>,
    pub continent_name: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub region_code: Option<String>,
    pub region_name: Option<String>,
    pub city_name: Option<String>,
    pub timezone: Option<String>,
    pub message: Option<String>,
    pub status_code: i32,
}

impl GeoData {
    pub fn new(ip_address: String) -> Self {
        GeoData {
            ip_address,
            latitude: 0.0,
            longitude: 0.0,
            continent_code: None,
            continent_name: None,
            country_code: None,
            country_name: None,
            region_code: None,
            region_name: None,
            city_name: None,
            timezone: None,
            message: None,
            status_code: 0,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GeoError<'a> {
    pub ip_address: &'a str,
    pub message: Option<&'a str>,
    pub status_code: i32,
}

impl<'a> GeoError<'a> {
    #[inline]
    pub fn new(ip_address: &'a str, message: Option<&'a str>) -> String {
        serde_json::to_string(&GeoError {
            ip_address,
            message,
            status_code: -1,
        })
        .unwrap()
    }
}
