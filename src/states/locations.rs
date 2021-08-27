use std::collections::HashMap;

use parking_lot::RwLock;

use crate::geolocation::{fetch, Location, IP};

pub struct LocationsState {
    pub locations: RwLock<HashMap<IP, Location>>,
}

impl Default for LocationsState {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationsState {
    pub fn new() -> Self {
        Self {
            locations: RwLock::new(HashMap::new()),
        }
    }

    pub fn resolve(&self, address: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            let locations = self.locations.read();

            if locations.get(&address).is_some() {
                return Ok(());
            }
        }

        let location = fetch(&address)?;

        {
            let mut locations = self.locations.write();
            locations.insert(address, location);
        }

        Ok(())
    }

    // pub fn get(&self, address: IP) -> Option<Location> {
    //     self.locations.read().get(&address).cloned()
    // }
}
