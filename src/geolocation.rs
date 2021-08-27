use serde::Deserialize;

const LOCATION_API_URL: &str = "https://ifconfig.based.computer/json";

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum IP {
    Local,
    Remote(String),
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

pub fn fetch(ip: &IP) -> Result<Location, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let mut request = client.get(LOCATION_API_URL);

    if let IP::Remote(ref ip) = ip {
        request = request.query(&[("ip", ip)])
    }

    let resp = request.send().unwrap().json::<Location>().unwrap();

    Ok(resp)
}
