use serde::Deserialize;

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
    pub async fn fetch(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<Location, Box<dyn std::error::Error>> {
        let mut request = client.get(format!("{}/json", url));

        if let Self::Remote(ref ip) = self {
            request = request.query(&[("ip", ip)])
        }

        let resp = request
            .send()
            .await
            .unwrap()
            .json::<Location>()
            .await
            .unwrap();

        Ok(resp)
    }
}
