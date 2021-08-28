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

impl IP {
    pub fn fetch(
        &self,
        client: &reqwest::blocking::Client,
    ) -> Result<Location, Box<dyn std::error::Error>> {
        let mut request = client.get(LOCATION_API_URL);

        if let Self::Remote(ref ip) = self {
            request = request.query(&[("ip", ip)])
        }

        let resp = request.send().unwrap().json::<Location>().unwrap();

        Ok(resp)
    }
}
