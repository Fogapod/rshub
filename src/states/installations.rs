use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use bytesize::ByteSize;

use anyhow::{anyhow, bail, Context};

use futures::stream::StreamExt;

use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::datatypes::{
    game_version::{DownloadUrl, GameVersion},
    installation::{Installation, InstallationKind},
    server::Address,
    value_sorted_map::ValueSortedMap,
};
use crate::states::app::{AppState, TaskResult};

pub struct InstallationsState {
    pub items: ValueSortedMap<GameVersion, Installation>,
    pub install_dir_error: Option<String>,
}

impl InstallationsState {
    pub async fn new(_: &AppConfig) -> Self {
        Self {
            items: ValueSortedMap::new(),
            install_dir_error: None,
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        self.spawn_installation_finder(app.clone()).await;
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn spawn_installation_finder(&mut self, app: Arc<AppState>) {
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
            .with_context(|| "Unable to read installation directory")?;

        let mut installations = installations.write().await;

        while let Some(fork_dirs) = dirs
            .next_entry()
            .await
            .with_context(|| "Unable to read fork list")?
        {
            let fork_path = fork_dirs.path();

            if !fork_path.is_dir() {
                continue;
            }

            let mut dirs = fs::read_dir(fork_path)
                .await
                .with_context(|| "Unable to read build directory")?;

            while let Some(build_dirs) = dirs
                .next_entry()
                .await
                .with_context(|| "Unable to read build list")?
            {
                let build_path = build_dirs.path();

                if !build_path.is_dir() {
                    continue;
                }

                let installation =
                    Installation::try_from_dir(&build_path)
                        .await
                        .with_context(|| {
                            format!("Unable to parse installation: {}", build_path.display())
                        })?;

                log::info!("found installation: {:?}", &installation);

                if let Some(existing) = installations.items.get(&installation.version.clone()) {
                    if matches!(
                        existing,
                        Installation {
                            kind: InstallationKind::Downloading { .. }
                                | InstallationKind::Unpacking,
                            ..
                        }
                    ) {
                        log::warn!("not overriding existing version {:?}", existing);
                        continue;
                    }
                }

                installations
                    .items
                    .insert(installation.version.clone(), installation);
            }
        }

        Ok(())
    }

    // intentionally blocking
    pub async fn refresh(&mut self, app: Arc<AppState>) {
        // remove everything except installing versions
        self.items.retain(|i| {
            matches!(
                i,
                Installation {
                    kind: InstallationKind::Downloading { .. } | InstallationKind::Unpacking,
                    ..
                }
            )
        });

        // grab versions from servers state
        for server in &app.servers.read().await.items {
            // these are the ones we filtered out
            if self.items.get(&server.version).is_some() {
                continue;
            }

            self.items.insert(
                server.version.clone(),
                Installation {
                    version: server.version.clone(),
                    kind: InstallationKind::Discovered,
                },
            );
        }

        self.spawn_installation_finder(app.clone()).await;
        app.events
            .read()
            .await
            .event("Refreshed installation list (still looking in file system)")
            .await;
    }

    pub async fn version_discovered(app: Arc<AppState>, version: &GameVersion) -> TaskResult {
        log::debug!("discovered: {}", version);

        let mut installations = app.installations.write().await;

        if let Some(existing) = installations.items.get(version).cloned() {
            if !matches!(&existing.kind, InstallationKind::Discovered) {
                log::debug!("not replacing existing {:?} with discovered", existing);

                return Ok(());
            }
        }

        app.events
            .read()
            .await
            .event(&format!("Discovered {}", version))
            .await;

        installations.items.insert(
            version.clone(),
            Installation {
                version: version.clone(),
                kind: InstallationKind::Discovered,
            },
        );

        Ok(())
    }

    pub async fn install(app: Arc<AppState>, version: GameVersion) -> TaskResult {
        let url = match &version.download {
            DownloadUrl::Valid(url) => url,
            DownloadUrl::Untrusted(bad) => {
                bail!("Not downloading (untrusted URL): `{}`", bad);
            }
            DownloadUrl::Invalid(bad) => {
                bail!("Not downloading (invalid URL): `{}`", bad);
            }
            DownloadUrl::Local => {
                bail!("Attempted to download installed version");
            }
        }
        .to_owned();

        app.events
            .read()
            .await
            .event(&format!("Downloading {}", version))
            .await;

        match app.installations.read().await.items.get(&version) {
            Some(Installation {
                kind: InstallationKind::Discovered,
                ..
            })
            | None => {}
            _ => {
                log::warn!("state desync: not discovered, ignoring install request");

                return Ok(());
            }
        }

        let installations = app.installations.clone();

        let response = app
            .client
            .get(url.clone())
            .send()
            .await
            .with_context(|| "Initial request failed")?;

        // TODO: handle downloads without known length
        let total = match response.content_length() {
            Some(total) => total,
            None => bail!("Unable to get content length"),
        };

        let mut path = app.config.dirs.installations_dir.clone();
        path.push(PathBuf::from(version.clone()));

        let path_parent = path.clone();

        path.push("data.zip");

        fs::create_dir_all(&path_parent)
            .await
            .with_context(|| "Unable to create installation folder")?;

        let mut file = fs::File::create(path.clone())
            .await
            .with_context(|| "Unable to create archive file")?;

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
                    bail!("Failed to read next chunk: {}", err);
                }
            };

            if let Err(err) = file.write(&chunk).await {
                bail!("Failed to write next chunk: {}", err);
            }

            progress += chunk.len();

            let mut installations = installations.write().await;
            let previous = installations.items.insert(
                version.clone(),
                Installation {
                    version: version.clone(),
                    kind: InstallationKind::Downloading {
                        progress: progress as u64,
                        total,
                    },
                },
            );

            if !matches!(
                previous,
                Some(Installation {
                    kind: InstallationKind::Downloading { .. },
                    ..
                }),
            ) {
                log::info!("aborting installation because installation state changed");

                previous.and_then(|previous| installations.items.insert(version.clone(), previous));

                if let Err(err) = fs::remove_dir_all(&path_parent).await {
                    log::error!(
                        "Unable to cleanup download directory {}: {}",
                        path_parent.display(),
                        err
                    );
                }

                return Ok(());
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

        app.events
            .read()
            .await
            .event(&format!("Extracting {}", version))
            .await;

        tokio::task::spawn_blocking(move || -> TaskResult {
            zip::read::ZipArchive::new(
                std::fs::File::open(path_cloned.clone())
                    .with_context(|| "Unable to read zip file")?,
            )
            .with_context(|| "Unable to decode zip file")?
            .extract(path_parent_cloned)
            .with_context(|| "Unable to extract zip file")
        })
        .await
        .with_context(|| "Task joining failed")?
        .with_context(|| "Archive decompression failed")?;

        if let Err(err) = fs::remove_file(&path).await {
            log::error!("Unable to cleanup zip file {}: {}", path.display(), err);
        }

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

        app.events
            .read()
            .await
            .event(&format!("Installed version {}", version))
            .await;

        Ok(())
    }

    pub async fn abort_installation(app: Arc<AppState>, version: GameVersion) -> TaskResult {
        let mut installations = app.installations.write().await;

        if matches!(
            installations.items.get(&version),
            Some(Installation {
                kind: InstallationKind::Downloading { .. } | InstallationKind::Unpacking,
                ..
            })
        ) {
            installations.items.insert(
                version.clone(),
                Installation {
                    version: version.clone(),
                    kind: InstallationKind::Discovered,
                },
            );

            app.events
                .read()
                .await
                .event(&format!("Aborted installation of {}", version))
                .await;
        } else {
            bail!("Nothing to abort");
        }

        Ok(())
    }

    pub async fn uninstall(app: Arc<AppState>, version: GameVersion) -> TaskResult {
        let mut path = app.config.dirs.installations_dir.clone();

        path.push(PathBuf::from(version.clone()));

        // lock in advance
        let mut installations = app.installations.write().await;

        match installations.items.get(&version) {
            Some(Installation {
                kind: InstallationKind::Installed { .. },
                ..
            }) => {}
            _ => {
                bail!("not installed, nothing to remove: {}", version);
            }
        }

        fs::remove_dir_all(path)
            .await
            .with_context(|| "Unable to remove build directory")?;

        installations.items.remove(&version);

        installations.refresh(app.clone()).await;

        app.events
            .read()
            .await
            .event(&format!("Uninstalled {}", version))
            .await;

        Ok(())
    }

    pub async fn launch(
        app: Arc<AppState>,
        version: GameVersion,
        address: Option<Address>,
    ) -> TaskResult {
        app.events
            .read()
            .await
            .event(&format!("Launching {}", version))
            .await;

        if !matches!(
            app.installations
                .read()
                .await
                .items
                .get(&version)
                .ok_or_else(|| anyhow!("desync: version not in installation list"))?,
            Installation {
                kind: InstallationKind::Installed { .. },
                ..
            }
        ) {
            Self::install(app.clone(), version.clone())
                .await
                .with_context(|| "Unable to install")?;
        } else {
            // https://github.com/unitystation/stationhub/blob/cebb9d45bff0a1c019852795a471068ba89d770a/UnitystationLauncher/Models/Installation.cs#L33-L104
            let mut path = app.config.dirs.installations_dir.clone();
            path.push(PathBuf::from(version.clone()));

            let mut exec_path = path.clone();

            #[cfg(target_family = "unix")]
            exec_path.push("Unitystation");
            #[cfg(target_os = "windows")]
            exec_path.push("Unitystation.exe");

            let mut command = Command::new(&exec_path);
            command.current_dir(&path);

            if let Some(address) = address {
                command
                    .arg("--server")
                    .arg(address.ip.to_string())
                    .arg("--port")
                    .arg(address.port.to_string())
                    // these are required for custom port and server because of bug
                    .arg("--refreshtoken")
                    .arg("gibberish")
                    .arg("--uid")
                    .arg("gibberish");
            }
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .with_context(|| "Unable to launch installation")?;
        }

        Ok(())
    }
}
