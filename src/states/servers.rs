use std::collections::HashMap;

use std::thread;
use std::time::Duration;

use parking_lot::RwLock;

use crate::constants::USER_AGENT;
use crate::datatypes::server::{Server, ServerListData};
use crate::geolocation::IP;
use crate::states::{SharedAppState, SharedLocationsState};
use crate::waitable_mutex::SharedWaitableMutex;

const SERVER_LIST_URL: &str = "https://api.unitystation.org/serverlist";

// use std::collections::hash_map::DefaultHasher;
// use std::hash::{Hash, Hasher};
//
// fn calculate_hash<T: Hash>(t: &T) -> u64 {
//     let mut s = DefaultHasher::new();
//     t.hash(&mut s);
//     s.finish()
// }

pub struct ServersState {
    pub items: RwLock<HashMap<String, Server>>,
}

impl Default for ServersState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServersState {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
        }
    }

    pub fn count(&self) -> usize {
        self.items.read().len()
    }

    pub fn update(
        &self,
        data: ServerListData,
        locations: SharedLocationsState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut servers = self.items.write();
        let mut existing = servers.clone();

        for sv in data.servers {
            if let Some(sv_existing) = servers.get_mut(&sv.ip) {
                existing.remove(&sv.ip);

                // if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                //     sv_existing.updated = true;
                // }

                sv_existing.offline = false;
                sv_existing.data = sv;
            } else {
                servers.insert(sv.ip.clone(), Server::new(&sv));
                locations.resolve(IP::Remote(sv.ip))?;
            }
        }

        for ip in existing.keys() {
            servers.get_mut(ip).unwrap().offline = true;
        }

        Ok(())
    }

    pub fn spawn_server_fetch_thread(
        interval: Duration,
        app: SharedAppState,
        stop_lock: SharedWaitableMutex<bool>,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("server_fetch".to_owned())
            .spawn(move || {
                let client = reqwest::blocking::Client::builder()
                    .user_agent(USER_AGENT)
                    .build()
                    .expect("creating client");

                if let Err(e) = app.locations.resolve(IP::Local) {
                    log::error!("error fetching local ip: {}", e);
                }

                let loop_body = move || {
                    let req = match client.get(SERVER_LIST_URL).send() {
                        Ok(req) => req,
                        Err(err) => {
                            log::error!("error creating request: {}", err);
                            return;
                        }
                    };
                    let req = match req.error_for_status() {
                        Ok(req) => req,
                        Err(err) => {
                            log::error!("bad status: {}", err);
                            return;
                        }
                    };
                    let resp = match req.json::<ServerListData>() {
                        Ok(resp) => resp,
                        Err(err) => {
                            log::error!("error decoding request: {}", err);
                            return;
                        }
                    };
                    if let Err(e) = app.servers.update(resp, app.locations.clone()) {
                        log::error!("error updating servers: {}", e);
                    }
                };
                loop {
                    loop_body();
                    stop_lock.wait_for(interval);
                    if stop_lock.get() {
                        break;
                    }
                }
            })
            .expect("failed to build thread")
    }
}
