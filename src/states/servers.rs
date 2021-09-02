use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::constants::SERVER_LIST_URL;
use crate::datatypes::game_version::{DownloadUrl, GameVersion};
use crate::datatypes::server::{Server, ServerListData};
use crate::datatypes::{geolocation::IP, installation::InstallationAction};
use crate::states::app::{AppState, TaskResult};
use crate::states::{InstallationsState, LocationsState};

// use std::collections::hash_map::DefaultHasher;
// use std::hash::{Hash, Hasher};
//
// fn calculate_hash<T: Hash>(t: &T) -> u64 {
//     let mut s = DefaultHasher::new();
//     t.hash(&mut s);
//     s.finish()
// }

pub struct ServersState {
    pub items: Vec<Server>,
    update_interval: Duration,
}

const DEBUG_GOOGOL_IP: &str = "8.8.8.8";

impl ServersState {
    pub async fn new(config: &AppConfig) -> Self {
        let items = vec![Server {
            name: "TEST SERVER PLEASE IGNORE".to_owned(),
            ip: IP::Remote(DEBUG_GOOGOL_IP.to_owned()),
            offline: true,
            version: GameVersion::new(
                "origin".to_owned(),
                9001.to_string(),
                DownloadUrl::new("https://evil.exploit"),
            ),
            fps: 42,
            time: "13:37".to_owned(),
            gamemode: "FFA".to_owned(),
            players: 7,
            map: "world".to_owned(),
            port: 22,
        }];

        Self {
            items,
            update_interval: Duration::from_secs(config.update_interval),
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        app.watch_task(tokio::task::spawn(Self::server_fetch_task(app.clone())))
            .await;

        let _ = app
            .locations
            .write()
            .await
            .resolve(IP::Remote(DEBUG_GOOGOL_IP.to_owned()));

        let _ = app
            .installations
            .read()
            .await
            .queue
            .send(InstallationAction::VersionDiscovered(GameVersion::new(
                "origin".to_owned(),
                9001.to_string(),
                DownloadUrl::new("https://evil.exploit"),
            )));
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn update(
        &mut self,
        data: ServerListData,
        locations: Arc<RwLock<LocationsState>>,
        installations: Arc<RwLock<InstallationsState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut existing: HashMap<IP, &mut Server> =
            self.items.iter_mut().map(|i| (i.ip.clone(), i)).collect();

        // avoid borrow issues in loop, if there is a better way tell me
        let mut created_servers: Vec<Server> = Vec::new();
        let mut existing_servers: Vec<IP> = Vec::new();

        for sv in data.servers {
            let ip = IP::Remote(sv.ip.clone());
            let version = GameVersion::from(sv.clone());

            if let Some(sv_existing) = existing.get_mut(&ip) {
                // version changed (download/build/fork)
                if sv_existing.version != version {
                    installations
                        .read()
                        .await
                        .queue
                        .send(InstallationAction::VersionDiscovered(version.clone()))
                        .unwrap();
                }

                existing_servers.push(ip);

                // if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                //     sv_existing.updated = true;
                // }

                sv_existing.offline = false;
                // sv_existing.data = sv;
            } else {
                let server = Server::new(ip.clone(), version, sv);

                installations
                    .read()
                    .await
                    .queue
                    .send(InstallationAction::VersionDiscovered(
                        server.version.clone(),
                    ))
                    .unwrap();

                created_servers.push(server);
                locations.write().await.resolve(ip).await?;
            }
        }

        for ip in existing_servers {
            existing.remove(&ip);
        }

        for sv in existing.values_mut() {
            sv.offline = true;
        }

        self.items.append(&mut created_servers);

        // TODO: custom sorts by each field
        // TODO: search by pattern
        // sorting priorities:
        //  - server is online
        //  - player count
        //  - server name
        // https://stackoverflow.com/a/40369685
        self.items.sort_by(|a, b| match a.offline.cmp(&b.offline) {
            Ordering::Equal => match a.players.cmp(&b.players).reverse() {
                Ordering::Equal => a.name.cmp(&b.name),
                other => other,
            },
            other => other,
        });

        Ok(())
    }

    async fn server_fetch_task(app: Arc<AppState>) -> TaskResult {
        let update_interval = app.servers.read().await.update_interval;

        if let Err(e) = app.locations.write().await.resolve(IP::Local).await {
            log::error!("error fetching local ip: {}", e);
        }

        loop {
            let req = match app.client.get(SERVER_LIST_URL).send().await {
                Ok(req) => req,
                Err(err) => {
                    log::error!("error creating request: {}", err);
                    todo!();
                }
            };
            let req = match req.error_for_status() {
                Ok(req) => req,
                Err(err) => {
                    log::error!("bad status: {}", err);
                    todo!();
                }
            };
            let resp = match req.json::<ServerListData>().await {
                Ok(resp) => resp,
                Err(err) => {
                    log::error!("error decoding request: {}", err);
                    todo!();
                }
            };
            if let Err(e) = app
                .servers
                .write()
                .await
                .update(resp, app.locations.clone(), app.installations.clone())
                .await
            {
                log::error!("error updating servers: {}", e);
            }

            tokio::time::sleep(update_interval).await;
        }
    }
}
