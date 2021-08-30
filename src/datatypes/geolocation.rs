use serde::Deserialize;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum IP {
    Local,
    Remote(String),
}

fn serde_unknown_f64_field() -> f64 {
    f64::NAN
}

fn serde_unknown_string_field() -> String {
    "unknown".to_owned()
}

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

impl Location {
    pub fn is_valid(&self) -> bool {
        !(self.longitude.is_nan() || self.latitude.is_nan())
    }
}
