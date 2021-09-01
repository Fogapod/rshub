use std::sync::Arc;

use tokio::fs;
use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::datatypes::{
    game_version::{DownloadUrl, GameVersion},
    installation::{Installation, InstallationAction, InstallationKind},
    value_sorted_map::ValueSortedMap,
};
use crate::states::app::{AppState, TaskResult};

pub struct InstallationsState {
    pub items: ValueSortedMap<GameVersion, Installation>,
    pub queue: mpsc::UnboundedSender<InstallationAction>,
    queue_recv: Option<mpsc::UnboundedReceiver<InstallationAction>>,
    pub install_dir_error: Option<String>,
}

impl InstallationsState {
    pub async fn new(_: &AppConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            items: ValueSortedMap::new(),
            queue: tx,
            queue_recv: Some(rx),
            install_dir_error: None,
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        let queue_recv = if let Some(queue_recv) = self.queue_recv.take() {
            queue_recv
        } else {
            log::error!("installation state: queue receiver already taken");
            return;
        };

        app.watch_task(tokio::task::spawn(Self::installation_handler_task(
            app.clone(),
            queue_recv,
        )))
        .await;

        self.spawn_installation_finder(app.clone()).await;
    }

    pub async fn spawn_installation_finder(&mut self, app: Arc<AppState>) {
        self.items.retain(|i| {
            !matches!(
                i,
                Installation {
                    kind: InstallationKind::Installed { .. },
                    ..
                }
            )
        });

        app.watch_task(tokio::task::spawn(Self::fs_installation_finder_task(
            app.config.clone(),
            app.installations.clone(),
        )))
        .await;
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
        app: Arc<AppState>,
        mut rx: mpsc::UnboundedReceiver<InstallationAction>,
    ) -> TaskResult {
        while let Some(action) = rx.recv().await {
            log::info!("installation action: {:?}", action);

            match action {
                InstallationAction::VersionDiscovered { new, old } => {
                    let mut installations = app.installations.write().await;

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

                    match app.installations.read().await.items.get(&version) {
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

                    let installations = app.installations.clone();
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
