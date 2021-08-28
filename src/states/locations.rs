use std::collections::HashMap;

use parking_lot::RwLock;

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::geolocation::{Location, IP};

pub struct LocationsState {
    pub items: RwLock<HashMap<IP, Location>>,
    client: reqwest::blocking::Client,
    geo_provider: String,
}

impl LocationsState {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            client: reqwest::blocking::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("creating client"),
            geo_provider: config.args.geo_provider.clone(),
        }
    }

    pub fn resolve(&self, address: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            let locations = self.items.read();

            if locations.get(&address).is_some() {
                return Ok(());
            }
        }

        let location = address.fetch(&self.client, &self.geo_provider)?;

        {
            let mut locations = self.items.write();
            locations.insert(address, location);
        }

        Ok(())
    }

    // pub fn get(&self, address: IP) -> Option<Location> {
    //     self.locations.read().get(&address).cloned()
    // }
}
