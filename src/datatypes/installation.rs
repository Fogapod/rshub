use std::io;
use std::path::Path;

use tokio::fs;

use crate::datatypes::server::{DownloadUrl, GameVersion};

#[derive(Debug)]
pub enum InstallationAction {
    VersionDiscovered {
        new: GameVersion,
        old: Option<GameVersion>,
    },
    Install(GameVersion),
    InstallCancel(GameVersion),
    Uninstall(GameVersion),
}

#[derive(Debug)]
pub enum InstallationKind {
    Discovered,
    Installed(u64),
    Corrupted(String),
    Downloading { progress: usize, total: usize },
    Unpacking,
}

#[derive(Debug)]
pub struct Installation {
    pub version: GameVersion,
    pub kind: InstallationKind,
}

impl Installation {
    pub async fn try_from_dir(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // FIXME: ugly hack
        let download = DownloadUrl::Invalid("[local]".to_owned());

        let fork = dir
            .file_name()
            .expect("expected non empty path")
            .to_str()
            .expect("bad directory ending")
            .to_owned();

        let mut dirs = fs::read_dir(dir).await.expect("reading build directory");

        let mut build_dir = None;

        while let Some(dir) = dirs
            .next_entry()
            .await
            .expect("reading build directory files")
        {
            let path = dir.path();
            if path.is_dir() {
                build_dir = Some(path);
            }
        }

        return Ok(if let Some(build_dir) = build_dir {
            Self {
                version: GameVersion {
                    fork,
                    build: build_dir
                        .file_name()
                        .expect("expected non empty path")
                        .to_str()
                        .expect("bad directory ending")
                        .to_owned(),
                    download,
                },
                kind: InstallationKind::Installed(
                    fs::metadata(build_dir).await.expect("read metadata").len(),
                ),
            }
        } else {
            Self {
                version: GameVersion {
                    fork,
                    build: "[not found]".to_owned(),
                    download,
                },
                kind: InstallationKind::Corrupted("No build directory".to_owned()),
            }
        });
    }
}
