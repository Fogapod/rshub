use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::RwLock;

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::server::{Server, ServerListData};
use crate::geolocation::IP;
use crate::states::LocationsState;
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
    update_interval: Duration,
}

const DEBUG_GOOGOL_IP: &str = "8.8.8.8";

impl ServersState {
    pub fn new(config: &AppConfig) -> Self {
        let items = RwLock::new(HashMap::new());

        items.write().insert(
            DEBUG_GOOGOL_IP.to_owned(),
            Server {
                offline: true,
                data: crate::datatypes::server::ServerData {
                    build: 0,
                    download: "none".to_owned(),
                    fork: "origin".to_owned(),
                    fps: 42,
                    time: "13:37".to_owned(),
                    gamemode: "FFA".to_owned(),
                    players: 7,
                    map: "world".to_owned(),
                    ip: DEBUG_GOOGOL_IP.to_owned(),
                    name: "googol".to_owned(),
                    port: 22,
                },
            },
        );
        Self {
            items,
            update_interval: Duration::from_secs(config.args.update_interval),
        }
    }

    pub fn count(&self) -> usize {
        self.items.read().len()
    }

    pub fn update(
        &self,
        data: ServerListData,
        locations: Arc<LocationsState>,
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
        &self,
        locations: Arc<LocationsState>,
        servers: Arc<ServersState>,
        stop_lock: SharedWaitableMutex<bool>,
    ) -> thread::JoinHandle<()> {
        let interval = self.update_interval;

        thread::Builder::new()
            .name("server_fetch".to_owned())
            .spawn(move || {
                let client = reqwest::blocking::Client::builder()
                    .user_agent(USER_AGENT)
                    .build()
                    .expect("creating client");

                if let Err(e) = locations.resolve(IP::Local) {
                    log::error!("error fetching local ip: {}", e);
                }

                // debugging
                let _ = locations.resolve(IP::Remote(DEBUG_GOOGOL_IP.to_owned()));

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
                    if let Err(e) = servers.update(resp, locations.clone()) {
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
