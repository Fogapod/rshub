use std::collections::HashMap;
use std::sync::Arc;

use tokio::fs;
use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::{
    installation::{Installation, InstallationAction, InstallationKind},
    server::{DownloadUrl, GameVersion},
};

pub struct InstallationsState {
    pub items: HashMap<GameVersion, Installation>,
    pub queue: mpsc::UnboundedSender<InstallationAction>,
}

impl InstallationsState {
    pub async fn new(
        app: &AppConfig,
        managed_tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    ) -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let instance = Arc::new(RwLock::new(Self {
            items: HashMap::new(),
            queue: tx,
        }));

        managed_tasks.push(tokio::task::spawn(Self::fs_installation_finder_task(
            app.clone(),
            instance.clone(),
        )));

        managed_tasks.push(tokio::task::spawn(Self::installation_handler_task(
            instance.clone(),
            rx,
        )));

        instance
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    async fn fs_installation_finder_task(app: AppConfig, installations: Arc<RwLock<Self>>) {
        log::debug!(
            "installation directory: {}",
            &app.dirs.installations_dir.display()
        );

        let mut dirs = fs::read_dir(app.dirs.installations_dir)
            .await
            .expect("reading installations directory");

        while let Some(dir) = dirs
            .next_entry()
            .await
            .expect("reading installations directory files")
        {
            let path = dir.path();

            if !path.is_dir() {
                log::warn!("not a directory: {}", path.display());

                continue;
            }

            let installation = Installation::try_from_dir(&path)
                .await
                .expect("scanning instllation");

            log::info!("found installation: {:?}", &installation);

            installations
                .write()
                .await
                .items
                .insert(installation.version.clone(), installation);
        }
    }

    async fn installation_handler_task(
        installations: Arc<RwLock<Self>>,
        mut rx: mpsc::UnboundedReceiver<InstallationAction>,
    ) {
        let _ = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        while let Some(action) = rx.recv().await {
            log::info!("installation action: {:?}", action);

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
                    let url = match &version.download {
                        DownloadUrl::Valid(url) => url,
                        DownloadUrl::Untrusted(bad) => {
                            log::warn!(
                                "not downloading untrusted content: {}",
                                String::from(bad.to_owned())
                            );
                            continue;
                        }
                        DownloadUrl::Invalid(bad) => {
                            log::warn!("not downloading invalid content: {}", bad);
                            continue;
                        }
                    };

                    log::info!("installing: {} ({})", &version, &String::from(url.clone()));

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
                InstallationAction::Uninstall(_) => {}
                InstallationAction::InstallCancel(_) => {}
            }
        }
    }
}
