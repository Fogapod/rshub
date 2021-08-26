use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time,
};

use parking_lot::{Condvar, Mutex, RwLock};

use crate::app::App;

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

pub fn ip_to_location(
    ip: IP,
    locations: &Arc<RwLock<HashMap<IP, Location>>>,
) -> Result<Location, Box<dyn std::error::Error>> {
    {
        let locations = locations.read();

        if let Some(&location) = locations.get(&ip) {
            return Ok(location);
        }
    }

    let client = reqwest::blocking::Client::new();

    let mut request = client.get(LOCATION_API_URL);

    if let IP::Remote(ref ip) = ip {
        request = request.query(&[("ip", ip)])
    }

    let resp = request.send().unwrap().json::<Location>().unwrap();

    {
        let mut locations = locations.write();
        locations.insert(ip, resp);
    }

    Ok(resp)
}
