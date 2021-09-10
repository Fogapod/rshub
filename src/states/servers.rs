use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;

use crate::config::AppConfig;
use crate::constants::SERVER_LIST_URL;
use crate::datatypes::game_version::{DownloadUrl, GameVersion};
use crate::datatypes::geolocation::IP;
use crate::datatypes::server::{Address, Server, ServerListJson};
use crate::states::app::{AppState, TaskResult};
use crate::states::versions::VersionsState;

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
        #[cfg(debug_assertions)]
        {
            let ip = IP::Remote("8.8.8.8".to_owned());
            let version = GameVersion {
                fork: "evil-exploit".to_owned(),
                build: 666.to_string(),
                download: DownloadUrl::new("http://evil.exploit"),
            };

            self.items.push(Server {
                name: "TEST SERVER PLEASE IGNORE".to_owned(),
                address: Address {
                    ip: ip.clone(),
                    port: 22,
                },
                offline: true,
                version: version.clone(),
                fps: 42,
                time: "13:37".to_owned(),
                gamemode: "FFA".to_owned(),
                players: 7,
                map: "world".to_owned(),
            });

            #[cfg(feature = "geolocation")]
            app.locations.write().await.resolve(&ip).await;

            let _ = VersionsState::version_discovered(Arc::clone(&app), &version).await;
        }

        if app.config.offline {
            return;
        }

        app.watch_task(tokio::task::spawn(Self::server_fetch_task(app.clone())))
            .await;
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn update(&mut self, app: Arc<AppState>, data: ServerListJson) {
        let mut previously_online: HashMap<Address, &mut Server> = self
            .items
            .iter_mut()
            .map(|i| (i.address.clone(), i))
            .collect();

        let mut created_servers: Vec<Server> = Vec::new();

        for sv in data.servers {
            let ip = IP::Remote(sv.ip.clone());
            let address = Address {
                ip: ip.clone(),
                port: sv.port,
            };
            let version = GameVersion::from(sv.clone());

            if let Some(known_server) = previously_online.remove(&address) {
                // version changed (download/build/fork)
                if known_server.version != version {
                    VersionsState::version_discovered(Arc::clone(&app), &version).await;
                    known_server.version = version;
                }

                known_server.update_from_json(&sv);

                known_server.offline = false;
            } else {
                #[cfg(feature = "geolocation")]
                app.locations.write().await.resolve(&ip).await;

                created_servers.push(Server::new(address, version.clone(), sv));

                VersionsState::version_discovered(Arc::clone(&app), &version).await;
            }
        }

        for sv in previously_online.values_mut() {
            sv.offline = true;
        }

        self.items.append(&mut created_servers);

        // TODO: pinned servers
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
    }

    async fn server_fetch_task(app: Arc<AppState>) -> TaskResult {
        let update_interval = app.servers.read().await.update_interval;

        #[cfg(feature = "geolocation")]
        app.locations.write().await.resolve(&IP::Local).await;

        async fn loop_body(app: Arc<AppState>) -> anyhow::Result<()> {
            let data = app
                .client
                .get(SERVER_LIST_URL)
                .send()
                .await
                .with_context(|| "sending server list request")?
                .error_for_status()?
                .json::<ServerListJson>()
                .await
                .with_context(|| "parsing server list response")?;

            app.servers.write().await.update(app.clone(), data).await;

            Ok(())
        }

        loop {
            if let Err(err) = loop_body(Arc::clone(&app)).await {
                app.events.read().await.error(err).await;
            }

            tokio::time::sleep(update_interval).await;
        }
    }
}
