use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

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

pub struct VersionsState {
    pub items: ValueSortedMap<GameVersion, Installation>,
    pub install_dir_error: Option<String>,
}

impl VersionsState {
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
            app.versions.clone(),
        )))
        .await;
    }

    async fn fs_installation_finder_task(
        app: AppConfig,
        versions: Arc<RwLock<Self>>,
    ) -> TaskResult {
        log::debug!(
            "installation directory: {}",
            &app.dirs.installations_dir.display()
        );

        let mut dirs = fs::read_dir(app.dirs.installations_dir)
            .await
            .with_context(|| "Unable to read installation directory")?;

        let mut versions = versions.write().await;

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

                if let Some(existing) = versions.items.get(&installation.version.clone()) {
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

                versions
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

    pub async fn version_discovered(app: Arc<AppState>, version: &GameVersion) {
        log::debug!("discovered: {}", version);

        let mut versions = app.versions.write().await;

        if let Some(existing) = versions.items.get(version).cloned() {
            if !matches!(&existing.kind, InstallationKind::Discovered) {
                log::debug!("not replacing existing {:?} with discovered", existing);

                return;
            }
        }

        app.events
            .read()
            .await
            .event(&format!("Discovered {}", version))
            .await;

        versions.items.insert(
            version.clone(),
            Installation {
                version: version.clone(),
                kind: InstallationKind::Discovered,
            },
        );
    }

    pub async fn install(app: Arc<AppState>, version: GameVersion) -> TaskResult {
        let url = match &version.download {
            DownloadUrl::Valid(url) => url,
            DownloadUrl::Untrusted(url) => {
                if !app.config.unchecked_downloads {
                    bail!("Not downloading (untrusted URL): `{}`", url);
                }

                url
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

        match app.versions.read().await.items.get(&version) {
            Some(Installation {
                kind: InstallationKind::Discovered,
                ..
            }) => {}
            Some(_) => {
                bail!("Attempted to download installed version");
            }
            _ => {
                bail!("state desync: not found, ignoring install request");
            }
        }

        let versions = app.versions.clone();

        let response = app
            .client
            .get(url.clone())
            .send()
            .await
            .with_context(|| "Initial request failed")?;

        let total = response.content_length();

        let build_home = app
            .config
            .dirs
            .installations_dir
            .join(PathBuf::from(version.clone()));

        let archive_file = build_home.join("data.zip");

        fs::create_dir_all(&build_home)
            .await
            .with_context(|| "Unable to create installation folder")?;

        log::warn!("{}", archive_file.display());
        let mut file = fs::File::create(archive_file.clone())
            .await
            .with_context(|| "Unable to create archive file")?;

        let mut stream = response.bytes_stream();

        let mut progress = 0;

        versions.write().await.items.insert(
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

            let mut versions = versions.write().await;
            let previous = versions.items.insert(
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

                previous.and_then(|previous| versions.items.insert(version.clone(), previous));

                if let Err(err) = fs::remove_dir_all(&build_home).await {
                    log::error!(
                        "Unable to cleanup download directory {}: {}",
                        build_home.display(),
                        err
                    );
                }

                return Ok(());
            }
        }

        versions.write().await.items.insert(
            version.clone(),
            Installation {
                version: version.clone(),
                kind: InstallationKind::Unpacking,
            },
        );

        drop(file);

        let path_cloned = archive_file.clone();
        let path_parent_cloned = build_home.clone();

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

        if let Err(err) = fs::remove_file(&archive_file).await {
            log::error!(
                "Unable to cleanup zip file {}: {}",
                archive_file.display(),
                err
            );
        }

        versions.write().await.items.insert(
            version.clone(),
            Installation {
                version: version.clone(),
                kind: InstallationKind::Installed {
                    size: Installation::get_folder_size(&build_home)
                        .await
                        .unwrap_or_default(),
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
        let mut versions = app.versions.write().await;

        if matches!(
            versions.items.get(&version),
            Some(Installation {
                kind: InstallationKind::Downloading { .. } | InstallationKind::Unpacking,
                ..
            })
        ) {
            versions.items.insert(
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
        let mut versions = app.versions.write().await;

        match versions.items.get(&version) {
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

        versions.items.remove(&version);

        versions.refresh(app.clone()).await;

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
            app.versions
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
        }

        // https://github.com/unitystation/stationhub/blob/cebb9d45bff0a1c019852795a471068ba89d770a/UnitystationLauncher/Models/Installation.cs#L33-L104
        let path = app
            .config
            .dirs
            .installations_dir
            .join(PathBuf::from(version.clone()));

        #[cfg(target_family = "unix")]
        let exec_path = path.join("Unitystation");
        #[cfg(target_os = "windows")]
        let exec_path = path.join("Unitystation.exe");
        #[cfg(not(any(target_family = "unix", target_os = "windows")))]
        bail!("Unsupported OS");

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

        Ok(())
    }
}
