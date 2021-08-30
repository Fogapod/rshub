use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::{
    installation::{Installation, InstallationAction, InstallationKind},
    server::GameVersion,
};

pub struct InstallationsState {
    pub items: HashMap<GameVersion, Installation>,
    pub queue: mpsc::UnboundedSender<InstallationAction>,
}

impl InstallationsState {
    pub async fn new(
        _: &AppConfig,
        managed_tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    ) -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let instance = Arc::new(RwLock::new(Self {
            items: HashMap::new(),
            queue: tx,
        }));

        managed_tasks.push(tokio::task::spawn(Self::installation_handler_task(
            instance.clone(),
            rx,
        )));

        instance
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn installation_handler_task(
        installations: Arc<RwLock<Self>>,
        mut rx: mpsc::UnboundedReceiver<InstallationAction>,
    ) {
        let _ = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        while let Some(action) = rx.recv().await {
            log::info!("instalaltion action: {:?}", action);

            match action {
                InstallationAction::VersionDiscovered { new, old } => {
                    let items = &mut installations.write().await.items;

                    if let Some(old) = old {
                        if let Some(existing) = items.get(&old) {
                            // we are replacing old with new only in case it was not
                            // installed or is not being installed
                            if let InstallationKind::Discovered = existing.kind {
                                items.remove(&old);
                            }
                        }
                    }

                    items.insert(
                        new.clone(),
                        Installation {
                            version: new,
                            kind: InstallationKind::Discovered,
                        },
                    );
                }
                InstallationAction::Install(version) => {
                    log::info!("installing: {:?}", version);

                    installations.write().await.items.insert(
                        version.clone(),
                        Installation {
                            version,
                            kind: InstallationKind::Downloading {
                                progress: 1,
                                total: 100,
                            },
                        },
                    );
                }
                _ => {}
            }
        }
    }
}
