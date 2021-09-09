use std::cmp::Ordering;
use std::io;
use std::path::Path;

use bytesize::ByteSize;

use anyhow::{Context, Result};

use tokio::fs;

use crate::datatypes::game_version::{DownloadUrl, GameVersion};

#[derive(Debug, Clone)]
pub enum InstallationKind {
    Discovered,
    Installed { size: ByteSize },
    Downloading { progress: u64, total: Option<u64> },
    Unpacking,
}

#[derive(Debug, Clone)]
pub struct Installation {
    pub version: GameVersion,
    pub kind: InstallationKind,
}

impl Installation {
    pub async fn try_from_dir(dir: &Path) -> Result<Self> {
        let build = dir
            .file_name()
            .unwrap()
            .to_str()
            .with_context(|| "Bad build directory name")?
            .to_owned();
        let fork = dir
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .with_context(|| "Bad fork directory name")?
            .to_owned();

        return Ok(Self {
            version: GameVersion {
                fork,
                build,
                download: DownloadUrl::Local,
            },
            kind: InstallationKind::Installed {
                size: ByteSize::b(Self::get_folder_size(dir).await.unwrap_or_default()),
            },
        });
    }

    // not recursive because async recursion is not possible without hacks
    pub async fn get_folder_size(path: &Path) -> io::Result<u64> {
        let mut result = 0;

        let mut dirs_to_check = vec![path.to_owned()];

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

impl PartialEq for Installation {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Installation {}

impl PartialEq<GameVersion> for Installation {
    fn eq(&self, version: &GameVersion) -> bool {
        self.version.eq(version)
    }
}

impl PartialOrd for Installation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Installation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.version.download.cmp(&other.version.download).reverse() {
            Ordering::Equal => self.version.cmp(&other.version),
            other => other,
        }
    }
}
