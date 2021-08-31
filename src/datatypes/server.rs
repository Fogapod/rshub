use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::datatypes::geolocation::IP;

fn deserialize_ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;

    let deserialized =
        T::deserialize(v).map_err(|e| log::error!("error deserializing field: {}", e));

    // this actually uses type default (String -> ""), so need to implement Deserialize
    // trait for more control over error handling
    Ok(deserialized.unwrap_or_default())
}

#[derive(Debug, Clone, Deserialize, Hash)]
#[serde(default)]
pub struct ServerData {
    #[serde(rename = "ServerName")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub name: String,
    #[serde(rename = "ForkName")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub fork: String,
    #[serde(rename = "BuildVersion")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub build: u32,
    #[serde(rename = "CurrentMap")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub map: String,
    #[serde(rename = "GameMode")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub gamemode: String,
    #[serde(rename = "IngameTime")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub time: String,
    #[serde(rename = "PlayerCount")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub players: u32,
    #[serde(rename = "ServerIP")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub ip: String,
    #[serde(rename = "ServerPort")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub port: u32,
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub fps: u32,

    #[cfg(target_os = "windows")]
    #[serde(rename = "WinDownload")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub download: String,

    #[cfg(target_os = "macos")]
    #[serde(rename = "OSXDownload")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub download: String,

    // should `target_family = "unix"` be used here?
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[serde(rename = "LinuxDownload")]
    #[serde(deserialize_with = "deserialize_ok_or_default")]
    pub download: String,
}

impl Default for ServerData {
    fn default() -> Self {
        let unknown_str = "unknown".to_owned();
        let unknown_u32 = 0;

        Self {
            name: unknown_str.clone(),
            fork: unknown_str.clone(),
            build: unknown_u32,
            map: unknown_str.clone(),
            gamemode: unknown_str.clone(),
            time: unknown_str.clone(),
            players: unknown_u32,
            ip: unknown_str.clone(),
            port: unknown_u32,
            fps: unknown_u32,
            download: unknown_str,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                if url == "https://evil.exploit" {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameVersion {
    pub fork: String,
    // expose string version for easier forward compatibility.
    // currently build numbers are numbers, but it might change:
    // https://github.com/unitystation/unitystation/issues/5089
    // u32 value are kept for fast cmp only
    pub build: String,
    pub(crate) build_u32: u32,
    pub download: DownloadUrl,
}

impl GameVersion {
    pub fn new(fork: String, build: String, download: DownloadUrl) -> Self {
        Self {
            fork,
            build: build.to_string(),
            build_u32: build.parse::<u32>().unwrap_or_default(),
            download,
        }
    }
}

impl From<ServerData> for GameVersion {
    fn from(data: ServerData) -> Self {
        let ServerData {
            fork,
            build,
            download,
            ..
        } = data;

        Self {
            fork,
            build: build.to_string(),
            build_u32: build,
            download: DownloadUrl::new(&download),
        }
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.fork, self.build)?;

        match self.download {
            DownloadUrl::Valid(_) | DownloadUrl::Local => Ok(()),
            DownloadUrl::Untrusted(_) => write!(f, " [untrusted download URL]"),
            DownloadUrl::Invalid(_) => write!(f, " [bad download URL]"),
        }
    }
}

impl PartialOrd for GameVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = match (&self.download, &other.download) {
            // local installations are the most important
            (DownloadUrl::Local, DownloadUrl::Local) => Ordering::Equal,
            (DownloadUrl::Local, _) => Ordering::Greater,
            // valid downloads
            (DownloadUrl::Valid(_), DownloadUrl::Valid(_)) => Ordering::Equal,
            (DownloadUrl::Valid(_), DownloadUrl::Untrusted(_) | DownloadUrl::Invalid(_)) => {
                Ordering::Greater
            }
            (DownloadUrl::Valid(_), DownloadUrl::Local) => Ordering::Less,
            // // untrusted downloads are less relevant
            (DownloadUrl::Untrusted(_), DownloadUrl::Untrusted(_)) => Ordering::Equal,
            (DownloadUrl::Untrusted(_), DownloadUrl::Invalid(_)) => Ordering::Greater,
            (DownloadUrl::Untrusted(_), DownloadUrl::Local | DownloadUrl::Valid(_)) => {
                Ordering::Less
            }
            // // invalid downloads are irrelevant
            (DownloadUrl::Invalid(_), DownloadUrl::Invalid(_)) => Ordering::Equal,
            (DownloadUrl::Invalid(_), _) => Ordering::Less,
        };

        if let Ordering::Equal = ordering {
            self.build_u32.cmp(&other.build_u32)
        } else {
            ordering
        }

        // implementation without internal u32 hint
        //
        // if let Ok(ordering) = self.build.parse::<u32>().and_then(|self_u32| {
        //     self.build
        //         .parse::<u32>()
        //         .map(|other_u32| self_u32.cmp(&other_u32))
        // }) {
        //     ordering
        // } else {
        //     Ordering::Equal
        // }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerListData {
    pub servers: Vec<ServerData>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub players: u32,
    pub version: GameVersion,
    pub ip: IP,
    pub port: u32,
    pub map: String,
    pub gamemode: String,
    pub time: String,
    pub fps: u32,
    // ui update skip optimization
    // pub updated: bool,
    pub offline: bool,
}

impl Server {
    pub fn new(ip: IP, version: GameVersion, data: ServerData) -> Self {
        let ServerData {
            name,
            map,
            gamemode,
            time,
            players,
            port,
            fps,
            ..
        } = data;

        Self {
            name: name.replace('\n', " "),
            map,
            gamemode,
            time,
            players,
            port,
            fps,
            ip,
            version,
            // updated: true,
            offline: false,
        }
    }
}
