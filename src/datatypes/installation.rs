use std::cmp::Ordering;
use std::path::Path;

use tokio::fs;

use crate::datatypes::game_version::{DownloadUrl, GameVersion};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationKind {
    Discovered,
    Installed { size: u64 },
    Downloading { progress: usize, total: usize },
    Unpacking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installation {
    pub version: GameVersion,
    pub kind: InstallationKind,
}

impl Installation {
    pub async fn try_from_dir(dir: &Path) -> Option<Self> {
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

        if let Some(build_dir) = build_dir {
            return Some(Self {
                version: GameVersion::new(
                    fork,
                    build_dir
                        .file_name()
                        .expect("expected non empty path")
                        .to_str()
                        .expect("bad directory ending")
                        .to_owned(),
                    DownloadUrl::Local,
                ),
                kind: InstallationKind::Installed {
                    size: fs::metadata(build_dir).await.expect("read metadata").len(),
                },
            });
        }

        log::warn!("fork {}: no build directory", fork);

        None
    }
}

impl PartialOrd for Installation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Installation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}
