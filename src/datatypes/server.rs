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

// because server names allow newlines
fn deserialize_remove_newlines<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = deserialize_ok_or_default(deserializer)?;

    Ok(s.replace('\n', " "))
}

#[derive(Debug, Clone, Deserialize, Hash)]
#[serde(default)]
pub struct ServerData {
    #[serde(rename = "ServerName")]
    #[serde(deserialize_with = "deserialize_remove_newlines")]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GameVersion {
    pub fork: String,
    pub build: u32,
    pub download: String,
}

impl GameVersion {
    pub fn new(data: ServerData) -> Self {
        let ServerData {
            fork,
            build,
            download,
            ..
        } = data;

        Self {
            fork,
            build,
            download,
        }
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
            name,
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
