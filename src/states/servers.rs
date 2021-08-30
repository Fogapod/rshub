use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::geolocation::IP;
use crate::datatypes::server::{Server, ServerListData};
use crate::states::LocationsState;

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
    pub items: HashMap<String, Server>,
    update_interval: Duration,
}

const DEBUG_GOOGOL_IP: &str = "8.8.8.8";

impl ServersState {
    pub async fn new(
        config: &AppConfig,
        locations: Arc<RwLock<LocationsState>>,
    ) -> Arc<RwLock<Self>> {
        let mut items = HashMap::new();

        items.insert(
            DEBUG_GOOGOL_IP.to_owned(),
            Server {
                ip: IP::Remote(DEBUG_GOOGOL_IP.to_owned()),
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
        let instance = Arc::new(RwLock::new(Self {
            items,
            update_interval: Duration::from_secs(config.args.update_interval),
        }));

        tokio::task::spawn(Self::server_fetch_task(instance.clone(), locations));

        instance
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn update(
        &mut self,
        data: ServerListData,
        locations: Arc<RwLock<LocationsState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut existing = self.items.clone();

        for sv in data.servers {
            if let Some(sv_existing) = self.items.get_mut(&sv.ip) {
                existing.remove(&sv.ip);

                // if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                //     sv_existing.updated = true;
                // }

                sv_existing.offline = false;
                sv_existing.data = sv;
            } else {
                self.items.insert(sv.ip.clone(), Server::new(&sv));
                locations.write().await.resolve(IP::Remote(sv.ip)).await?;
            }
        }

        for ip in existing.keys() {
            self.items.get_mut(ip).unwrap().offline = true;
        }

        Ok(())
    }

    pub async fn server_fetch_task(
        servers: Arc<RwLock<ServersState>>,
        locations: Arc<RwLock<LocationsState>>,
    ) {
        let update_interval = servers.read().await.update_interval;

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        if let Err(e) = locations.write().await.resolve(IP::Local).await {
            log::error!("error fetching local ip: {}", e);
        }

        // debugging
        let _ = locations
            .write()
            .await
            .resolve(IP::Remote(DEBUG_GOOGOL_IP.to_owned()));

        loop {
            let req = match client.get(SERVER_LIST_URL).send().await {
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
            let resp = match req.json::<ServerListData>().await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("error decoding request: {}", err);
                    return;
                }
            };
            if let Err(e) = servers.write().await.update(resp, locations.clone()).await {
                log::error!("error updating servers: {}", e);
            }

            tokio::time::sleep(update_interval).await;
        }
    }
}
