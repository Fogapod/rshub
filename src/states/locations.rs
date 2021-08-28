use std::collections::HashMap;

use parking_lot::RwLock;

use crate::constants::USER_AGENT;
use crate::geolocation::{Location, IP};

pub struct LocationsState {
    pub items: RwLock<HashMap<IP, Location>>,
    client: reqwest::blocking::Client,
}

impl Default for LocationsState {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationsState {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            client: reqwest::blocking::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("creating client"),
        }
    }

    pub fn resolve(&self, address: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            let locations = self.items.read();

            if locations.get(&address).is_some() {
                return Ok(());
            }
        }

        let location = address.fetch(&self.client)?;

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
