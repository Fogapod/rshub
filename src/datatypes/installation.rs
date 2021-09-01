use std::cmp::Ordering;
use std::io;
use std::path::{Path, PathBuf};

use bytesize::ByteSize;

use tokio::fs;

use crate::datatypes::game_version::{DownloadUrl, GameVersion};

#[derive(Debug)]
pub enum InstallationAction {
    VersionDiscovered {
        new: GameVersion,
        old: Option<GameVersion>,
    },
    Install(GameVersion),
    AbortInstall(GameVersion),
    Uninstall(GameVersion),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationKind {
    Discovered,
    Installed { size: ByteSize },
    Downloading { progress: u64, total: u64 },
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
                    size: ByteSize::b(
                        Self::get_folder_size(build_dir)
                            .await
                            .expect("get directory size"),
                    ),
                },
            });
        }

        log::warn!("fork {}: no build directory", fork);

        None
    }

    // not recursive because async recursion is not possible without hacks
    async fn get_folder_size(path: PathBuf) -> io::Result<u64> {
        let mut result = 0;

        let mut dirs_to_check = vec![path];

        while let Some(next_dir) = dirs_to_check.pop() {
            let mut stream = fs::read_dir(next_dir).await?;

            while let Some(dir) = stream.next_entry().await? {
                let path = dir.path();

                if path.is_file() {
                    result += fs::metadata(path).await?.len();
                } else if path.is_dir() {
                    dirs_to_check.push(path);
                }
            }
        }

        Ok(result)
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
