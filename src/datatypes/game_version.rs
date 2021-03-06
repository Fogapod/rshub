use std::cmp::Ordering;
use std::fmt;
use std::path::PathBuf;

use crate::constants::DEFAULT_CDN_DOMAIN;
use crate::datatypes::server::ServerJson;

#[derive(Debug, Clone)]
pub enum DownloadUrl {
    Valid(reqwest::Url),
    Untrusted(reqwest::Url),
    Invalid(String),
    Local,
}

impl DownloadUrl {
    pub fn new(url: &str) -> Self {
        match reqwest::Url::parse(url) {
            Ok(parsed) => {
                // https://github.com/unitystation/stationhub/blob/cebb9d45bff0a1c019852795a471068ba89d770a/UnitystationLauncher/Models/Server.cs#L37-L57
                if parsed.scheme() != "https"
                    || parsed.host().map(|h| h.to_string()) != Some(DEFAULT_CDN_DOMAIN.to_owned())
                {
                    Self::Untrusted(parsed)
                } else {
                    Self::Valid(parsed)
                }
            }
            Err(e) => {
                log::debug!("error parsing URL: {}", e);
                Self::Invalid(url.to_owned())
            }
        }
    }
}

impl PartialEq for DownloadUrl {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for DownloadUrl {}

impl PartialOrd for DownloadUrl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DownloadUrl {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, &other) {
            // local installations are the most important
            (DownloadUrl::Local, DownloadUrl::Local) => Ordering::Equal,
            (DownloadUrl::Local, _) => Ordering::Greater,
            // valid downloads
            (DownloadUrl::Valid(_), DownloadUrl::Valid(_)) => Ordering::Equal,
            (DownloadUrl::Valid(_), DownloadUrl::Untrusted(_) | DownloadUrl::Invalid(_)) => {
                Ordering::Greater
            }
            (DownloadUrl::Valid(_), DownloadUrl::Local) => Ordering::Less,
            // untrusted downloads are less relevant
            (DownloadUrl::Untrusted(_), DownloadUrl::Untrusted(_)) => Ordering::Equal,
            (DownloadUrl::Untrusted(_), DownloadUrl::Invalid(_)) => Ordering::Greater,
            (DownloadUrl::Untrusted(_), DownloadUrl::Local | DownloadUrl::Valid(_)) => {
                Ordering::Less
            }
            // invalid downloads are irrelevant
            (DownloadUrl::Invalid(_), DownloadUrl::Invalid(_)) => Ordering::Equal,
            (DownloadUrl::Invalid(_), _) => Ordering::Less,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameVersion {
    pub fork: String,
    pub build: String,
    pub download: DownloadUrl,
}

impl From<ServerJson> for GameVersion {
    fn from(data: ServerJson) -> Self {
        let ServerJson {
            fork,
            build,
            download,
            ..
        } = data;

        Self {
            // replace / for security reasons, just in case
            fork: fork.replace('/', ""),
            build: build.to_string(),
            download: DownloadUrl::new(&download),
        }
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.fork, self.build)?;

        match self.download {
            DownloadUrl::Valid(_) | DownloadUrl::Local => Ok(()),
            DownloadUrl::Untrusted(_) => write!(f, " [untrusted download]"),
            DownloadUrl::Invalid(_) => write!(f, " [bad download]"),
        }
    }
}

impl PartialEq for GameVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for GameVersion {}

impl PartialOrd for GameVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.fork.cmp(&other.fork) {
            Ordering::Equal => self.build.cmp(&other.build),
            other => other,
        }
    }
}

impl From<GameVersion> for PathBuf {
    fn from(version: GameVersion) -> Self {
        PathBuf::from(version.fork).join(version.build)
    }
}
