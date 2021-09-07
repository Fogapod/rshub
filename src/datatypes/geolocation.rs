use std::fmt;

#[cfg(feature = "geolocation")]
use serde::Deserialize;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum IP {
    #[cfg(feature = "geolocation")]
    Local,
    Remote(String),
}

#[cfg(feature = "geolocation")]
fn serde_unknown_f64_field() -> f64 {
    f64::NAN
}

#[cfg(feature = "geolocation")]
fn serde_unknown_string_field() -> String {
    "unknown".to_owned()
}

#[cfg(feature = "geolocation")]
#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(default = "serde_unknown_f64_field")]
    pub longitude: f64,
    #[serde(default = "serde_unknown_f64_field")]
    pub latitude: f64,
    #[serde(default = "serde_unknown_string_field")]
    pub country: String,
    #[serde(default = "serde_unknown_string_field")]
    pub city: String,
}

#[cfg(feature = "geolocation")]
impl Location {
    pub fn is_valid(&self) -> bool {
        !(self.longitude.is_nan() || self.latitude.is_nan())
    }
}

impl fmt::Display for IP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "geolocation")]
            Self::Local => write!(f, "127.0.0.1"),
            Self::Remote(ip) => write!(f, "{}", ip),
        }
    }
}
