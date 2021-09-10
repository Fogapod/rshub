mod draw;
mod hotkeys;
mod input;
mod state;
mod tasks;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tui::widgets::TableState;

use crate::datatypes::game_version::{DownloadUrl, GameVersion};
use crate::datatypes::geolocation::IP;
use crate::datatypes::server::{Address, Server, ServerListJson};
use crate::states::{AppState, StatelessList};
use crate::views::Name;

use state::State;

use crate::views::AppView;

pub struct Servers {
    state: Arc<RwLock<State>>,
    selection: StatelessList<TableState>,
}

impl Servers {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(State::new())),
            selection: StatelessList::new(TableState::default(), false),
        }
    }

    pub async fn run(&self, app: Arc<AppState>) {
        #[cfg(debug_assertions)]
        {
            let ip = IP::Remote("8.8.8.8".to_owned());
            let version = GameVersion {
                fork: "evil-exploit".to_owned(),
                build: 666.to_string(),
                download: DownloadUrl::new("http://evil.exploit"),
            };

            self.state.write().await.items.push(Server {
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

            // let _ = VersionsState::version_discovered(Arc::clone(&app), &version).await;
        }

        if app.config.offline {
            return;
        }

        app.watch_task(tokio::task::spawn(tasks::server_fetch_task(app.clone())))
            .await;
    }

    pub async fn count(&self) -> usize {
        self.state.read().await.items.len()
    }

    pub async fn update(&self, app: Arc<AppState>, data: ServerListJson) {
        let mut items = self.state.write().await.items;

        let mut previously_online: HashMap<Address, &mut Server> =
            items.iter_mut().map(|i| (i.address.clone(), i)).collect();

        let mut created_servers: Vec<Server> = Vec::new();

        for sv in data.servers {
            let ip = IP::Remote(sv.ip.clone());
            let address = Address {
                ip: ip.clone(),
                port: sv.port,
            };
            let version = GameVersion::from(sv.clone());

            if let Some(known_server) = previously_online.remove(&address) {
                // download/build/fork changed
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

        items.append(&mut created_servers);

        // TODO: pinned servers
        // TODO: custom sorts by each field
        // TODO: search by pattern
        // sorting priorities:
        //  - server is online
        //  - player count
        //  - server name
        // https://stackoverflow.com/a/40369685
        items.sort_by(|a, b| match a.offline.cmp(&b.offline) {
            Ordering::Equal => match a.players.cmp(&b.players).reverse() {
                Ordering::Equal => a.name.cmp(&b.name),
                other => other,
            },
            other => other,
        });
    }
}

impl Default for Servers {
    fn default() -> Self {
        Self::new()
    }
}

impl AppView for Servers {}

impl Name for Servers {
    fn name(&self) -> String {
        "Server List".to_owned()
    }
}
