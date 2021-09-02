use std::path::PathBuf;
use std::sync::Arc;

use bytesize::ByteSize;

use futures::stream::StreamExt;

use tokio::fs;
use tokio::io::AsyncWriteExt;
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

    pub fn count(&self) -> usize {
        self.items.len()
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

        let mut installations = installations.write().await;

        while let Some(fork_dirs) = dirs
            .next_entry()
            .await
            .expect("reading installations directory files")
        {
            let fork_path = fork_dirs.path();

            if !fork_path.is_dir() {
                log::warn!("not a directory: {}", fork_path.display());

                continue;
            }

            let mut dirs = fs::read_dir(fork_path)
                .await
                .expect("reading installations directory");

            while let Some(build_dirs) = dirs
                .next_entry()
                .await
                .expect("reading fork directory files")
            {
                let build_path = build_dirs.path();

                if !build_path.is_dir() {
                    log::warn!("not a directory: {}", build_path.display());

                    continue;
                }

                let installation = match Installation::try_from_dir(&build_path).await {
                    Some(installation) => installation,
                    None => continue,
                };

                log::info!("found installation: {:?}", &installation);

                installations
                    .items
                    .insert(installation.version.clone(), installation);
            }
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
                InstallationAction::VersionDiscovered(version) => {
                    Self::version_discovered(app.clone(), version).await;
                }
                InstallationAction::Install(version) => Self::install(app.clone(), version).await,
                InstallationAction::AbortInstall(version) => {
                    Self::abort_installation(app.clone(), version).await
                }
                InstallationAction::Uninstall(version) => {
                    Self::uninstall(app.clone(), version).await
                }
            }
        }

        Ok(())
    }

    async fn version_discovered(app: Arc<AppState>, version: GameVersion) {
        let mut installations = app.installations.write().await;

        if let Some(existing) = installations.items.get(&version).cloned() {
            if !matches!(&existing.kind, InstallationKind::Discovered) {
                return;
            }
        }

        installations.items.insert(
            version.clone(),
            Installation {
                version,
                kind: InstallationKind::Discovered,
            },
        );
    }

    async fn install(app: Arc<AppState>, version: GameVersion) {
        let url = match &version.download {
            DownloadUrl::Valid(url) => url,
            DownloadUrl::Untrusted(bad) => {
                log::warn!(
                    "not downloading untrusted content: {}",
                    String::from(bad.to_owned())
                );
                return;
            }
            DownloadUrl::Invalid(bad) => {
                log::warn!("not downloading invalid content: {}", bad);
                return;
            }
            DownloadUrl::Local => {
                log::warn!("attempted to download local version");
                return;
            }
        }
        .to_owned();

        match app.installations.read().await.items.get(&version) {
            Some(Installation {
                kind: InstallationKind::Discovered,
                ..
            })
            | None => {}
            _ => {
                log::warn!("installation state: not discovered, ignoring install request");

                return;
            }
        }

        log::info!("installing: {} ({})", &version, &String::from(url.clone()));

        let installations = app.installations.clone();

        tokio::spawn(async move {
            let response = app
                .client
                .get(url.clone())
                .send()
                .await
                .expect("download failed");

            let total = response
                .content_length()
                .expect("TODO: missing content length");

            let mut path = app.config.dirs.installations_dir.clone();
            path.push(PathBuf::from(version.clone()));

            let path_parent = path.clone();

            path.push("data.zip");

            fs::create_dir_all(&path_parent)
                .await
                .expect("TODO: folder creation failed");

            let mut file = fs::File::create(path.clone())
                .await
                .expect("TODO: file creation failed");

            let mut stream = response.bytes_stream();

            let mut progress = 0;

            installations.write().await.items.insert(
                version.clone(),
                Installation {
                    version: version.clone(),
                    kind: InstallationKind::Downloading { progress: 0, total },
                },
            );

            while let Some(item) = stream.next().await {
                let chunk = match item {
                    Ok(chunk) => chunk,
                    Err(err) => {
                        log::error!("failed to read next chunk: {}", err);
                        return;
                    }
                };

                if let Err(err) = file.write(&chunk).await {
                    log::error!("failed to write chunk: {}", err);
                    return;
                }

                progress += chunk.len();

                let mut installations = installations.write().await;
                if let Some(previous) = installations.items.insert(
                    version.clone(),
                    Installation {
                        version: version.clone(),
                        kind: InstallationKind::Downloading {
                            progress: progress as u64,
                            total,
                        },
                    },
                ) {
                    if !matches!(
                        previous,
                        Installation {
                            kind: InstallationKind::Downloading { .. },
                            ..
                        },
                    ) {
                        log::info!("aborting installation because installation state changed");

                        installations.items.insert(version.clone(), previous);

                        fs::remove_dir_all(&path_parent)
                            .await
                            .expect("failed to cleanup");

                        return;
                    }
                }
            }

            installations.write().await.items.insert(
                version.clone(),
                Installation {
                    version: version.clone(),
                    kind: InstallationKind::Unpacking,
                },
            );

            drop(file);

            let path_cloned = path.clone();
            let path_parent_cloned = path_parent.clone();

            tokio::task::spawn_blocking(move || {
                zip::read::ZipArchive::new(
                    std::fs::File::open(path_cloned.clone()).expect("cannot open zip file"),
                )
                .expect("cannot open archive")
                .extract(path_parent_cloned)
                .expect("cannot extract")
            })
            .await
            .expect("something broke");

            fs::remove_file(&path).await.expect("cannot delete zip");

            installations.write().await.items.insert(
                version.clone(),
                Installation {
                    version: version.clone(),
                    kind: InstallationKind::Installed {
                        size: ByteSize::b(
                            Installation::get_folder_size(&path_parent)
                                .await
                                .unwrap_or_default(),
                        ),
                    },
                },
            );
        });
    }

    async fn abort_installation(app: Arc<AppState>, version: GameVersion) {
        app.installations.write().await.items.insert(
            version.clone(),
            Installation {
                version,
                kind: InstallationKind::Discovered,
            },
        );
    }

    async fn uninstall(app: Arc<AppState>, version: GameVersion) {
        let mut path = app.config.dirs.installations_dir.clone();

        path.push(PathBuf::from(version.clone()));

        // lock in advance
        let mut installations = app.installations.write().await;

        match installations.items.get(&version) {
            Some(Installation {
                kind: InstallationKind::Installed { .. },
                ..
            }) => {}
            other => {
                log::error!("not installed, nothing to remove: {} {:?}", version, other);
                return;
            }
        }

        fs::remove_dir_all(path)
            .await
            .expect("TODO: removal failed");

        installations.items.remove(&version);
    }
}
