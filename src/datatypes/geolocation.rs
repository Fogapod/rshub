use std::convert::TryFrom;
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
fn serde_unknown_string_field() -> String {
    "unknown".to_owned()
}

#[cfg(feature = "geolocation")]
#[derive(Deserialize)]
pub struct LocationJson {
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    #[serde(default = "serde_unknown_string_field")]
    pub country: String,
    #[serde(default = "serde_unknown_string_field")]
    pub city: String,
}

#[derive(Debug)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
    pub country: String,
    pub city: String,
}

impl TryFrom<&LocationJson> for Location {
    type Error = &'static str;

    fn try_from(value: &LocationJson) -> Result<Self, Self::Error> {
        let LocationJson {
            longitude,
            latitude,
            country,
            city,
        } = value;

        let (longitude, latitude) = match (longitude, latitude) {
            (Some(lo), Some(la)) => (*lo, *la),
            _ => {
                return Err("Missing longitude or latitude");
            }
        };

        Ok(Self {
            longitude,
            latitude,
            country: country.clone(),
            city: city.clone(),
        })
    }
}

impl fmt::Display for IP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "geolocation")]
            Self::Local => write!(f, "localhost"),
            Self::Remote(ip) => write!(f, "{}", ip),
        }
    }
}
