use std::sync::Arc;

use tokio::fs;
use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::{
    installation::{Installation, InstallationAction, InstallationKind},
    server::{DownloadUrl, GameVersion},
    value_sorted_map::ValueSortedMap,
};
use crate::states::app::{TaskQueue, TaskResult};

pub struct InstallationsState {
    pub items: ValueSortedMap<GameVersion, Installation>,
    pub queue: mpsc::UnboundedSender<InstallationAction>,
    pub install_dir_error: Option<String>,
    tasks: TaskQueue,
}

impl InstallationsState {
    pub async fn new(app: &AppConfig, tasks: TaskQueue) -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let instance = Arc::new(RwLock::new(Self {
            items: ValueSortedMap::new(),
            queue: tx,
            install_dir_error: None,
            tasks: tasks.clone(),
        }));

        let tasks = tasks.read().await;

        tasks
            .send(tokio::task::spawn(Self::installation_handler_task(
                instance.clone(),
                rx,
            )))
            .expect("spawn installation finder task");

        Self::spawn_installation_finder(app, instance.clone()).await;

        instance
    }

    pub async fn spawn_installation_finder(app: &AppConfig, installations: Arc<RwLock<Self>>) {
        let mut self_ = installations.write().await;

        self_.items.retain(|i| {
            !matches!(
                i,
                Installation {
                    kind: InstallationKind::Installed { .. },
                    ..
                }
            )
        });

        self_
            .tasks
            .read()
            .await
            .send(tokio::task::spawn(Self::fs_installation_finder_task(
                app.to_owned(),
                installations.clone(),
            )))
            .expect("spawn installation handler task");
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    async fn fs_installation_finder_task(
        app: AppConfig,
        installations: Arc<RwLock<Self>>,
    ) -> TaskResult {
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

            let installation = match Installation::try_from_dir(&path).await {
                Some(installation) => installation,
                None => continue,
            };

            log::info!("found installation: {:?}", &installation);

            installations
                .write()
                .await
                .items
                .insert(installation.version.clone(), installation);
        }

        Ok(())
    }

    async fn installation_handler_task(
        installations: Arc<RwLock<Self>>,
        mut rx: mpsc::UnboundedReceiver<InstallationAction>,
    ) -> TaskResult {
        let _ = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        while let Some(action) = rx.recv().await {
            log::info!("installation action: {:?}", action);

            match action {
                InstallationAction::VersionDiscovered { new, old } => {
                    let mut installations = installations.write().await;

                    if let Some(old_version) = old {
                        if let Some(existing) = installations.items.get(&old_version).cloned() {
                            // we are replacing old with new only in case it was not
                            // installed or is not being installed
                            if let InstallationKind::Discovered = &existing.kind {
                                installations.items.remove_value(&existing);
                            }
                        }
                    }

                    installations.items.insert(
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
                        DownloadUrl::Local => {
                            log::warn!("attempted to download local version");
                            continue;
                        }
                    };

                    match installations.read().await.items.get(&version) {
                        Some(Installation {
                            kind: InstallationKind::Discovered,
                            ..
                        })
                        | None => {}
                        _ => {
                            log::warn!(
                                "installation state: not discovered, ignoring install request"
                            );

                            continue;
                        }
                    }

                    log::info!("installing: {} ({})", &version, &String::from(url.clone()));

                    let installations = installations.clone();
                    tokio::spawn(async move {
                        for progress in 0..100 {
                            let version = version.clone();
                            installations.write().await.items.insert(
                                version.clone(),
                                Installation {
                                    version,
                                    kind: InstallationKind::Downloading {
                                        progress,
                                        total: 100,
                                    },
                                },
                            );

                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }

                        installations.write().await.items.insert(
                            version.clone(),
                            Installation {
                                version,
                                kind: InstallationKind::Discovered,
                            },
                        )
                    });
                }
                InstallationAction::Uninstall(_) => {}
                InstallationAction::InstallCancel(_) => {}
            }
        }

        Ok(())
    }
}
