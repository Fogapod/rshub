use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::config::AppConfig;
use crate::constants::SERVER_LIST_URL;
use crate::datatypes::game_version::{DownloadUrl, GameVersion};
use crate::datatypes::geolocation::IP;
use crate::datatypes::server::{Server, ServerListData};
use crate::states::app::{AppState, TaskResult};
use crate::states::installations::VersionOperation;

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

impl ServersState {
    pub async fn new(config: &AppConfig) -> Self {
        Self {
            items: Vec::new(),
            update_interval: Duration::from_secs(config.update_interval),
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        app.watch_task(tokio::task::spawn(Self::server_fetch_task(app.clone())))
            .await;

        #[cfg(debug_assertions)]
        {
            let ip = IP::Remote("8.8.8.8".to_owned());
            let version = GameVersion {
                fork: "evil-exploit".to_owned(),
                build: 666.to_string(),
                download: DownloadUrl::new("https://evil.exploit"),
            };

            self.items.push(Server {
                name: "TEST SERVER PLEASE IGNORE".to_owned(),
                ip: ip.clone(),
                offline: true,
                version: version.clone(),
                fps: 42,
                time: "13:37".to_owned(),
                gamemode: "FFA".to_owned(),
                players: 7,
                map: "world".to_owned(),
                port: 22,
            });

            let _ = app.locations.write().await.resolve(ip);

            app.installations
                .read()
                .await
                .operation(app.clone(), VersionOperation::Discover(version))
                .await;
        }
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn update(
        &mut self,
        app: Arc<AppState>,
        data: ServerListData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut previously_online: HashMap<IP, &mut Server> =
            self.items.iter_mut().map(|i| (i.ip.clone(), i)).collect();

        let mut created_servers: Vec<Server> = Vec::new();

        for sv in data.servers {
            let ip = IP::Remote(sv.ip.clone());
            let version = GameVersion::from(sv.clone());

            if let Some(known_server) = previously_online.remove(&ip) {
                // version changed (download/build/fork)
                if known_server.version != version {
                    app.installations
                        .read()
                        .await
                        .operation(app.clone(), VersionOperation::Discover(version.clone()))
                        .await;
                }

                // if calculate_hash(&known_server.data) != calculate_hash(&sv) {
                //     known_server.updated = true;
                // }

                known_server.offline = false;
                // known_server.data = sv;
            } else {
                created_servers.push(Server::new(ip.clone(), version.clone(), sv));

                app.installations
                    .read()
                    .await
                    .operation(app.clone(), VersionOperation::Discover(version))
                    .await;

                app.locations.write().await.resolve(ip).await?;
            }
        }

        for sv in previously_online.values_mut() {
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
            if let Err(e) = app.servers.write().await.update(app.clone(), resp).await {
                log::error!("error updating servers: {}", e);
            }

            tokio::time::sleep(update_interval).await;
        }
    }
}
