use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Clap;

#[cfg(feature = "geolocation")]
use crate::constants::DEFAULT_GEO_PROVIDER_URL;

// thanks kalmari
fn greater_than_5(s: &str) -> Result<u64, String> {
    let min_value = 5;

    let v = s.parse::<u64>().map_err(|e| e.to_string())?;
    if v < min_value {
        Err(format!("Value must be >= {}", min_value))
    } else {
        Ok(v)
    }
}

#[derive(Clap, Debug)]
#[clap(version = clap::crate_version!(), about = "UnityStation server hub")]
struct CliArgs {
    /// Log file path
    #[clap(short, long)]
    log_file: Option<PathBuf>,
    /// Server list update interval, in seconds (must be >= 5)
    #[clap(short, long, default_value = "20", parse(try_from_str = greater_than_5))]
    update_interval: u64,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: u32,
    /// Geolocation provider (ifconfig.co compatible)
    #[cfg(feature = "geolocation")]
    #[clap(long, default_value = DEFAULT_GEO_PROVIDER_URL)]
    geo_provider: reqwest::Url,
    /// Offline mode
    #[clap(long)]
    offline: bool,
    /// Disable download URL verification
    #[clap(long)]
    unchecked_downloads: bool,
}

#[derive(Debug, Clone)]
pub struct AppDirs {
    pub log_file: PathBuf,

    pub data_dir: PathBuf,
    pub installations_dir: PathBuf,
}

impl AppDirs {
    fn new(log_file: Option<PathBuf>) -> Result<Self, io::Error> {
        let data_dir = Self::get_data_dir()?;

        Ok(Self {
            log_file: log_file.unwrap_or_else(|| Self::default_log_path(&data_dir)),
            installations_dir: Self::get_installations_dir(data_dir.clone())?,
            data_dir,
        })
    }

    fn get_data_dir() -> Result<PathBuf, io::Error> {
        let mut data = dirs_next::data_dir().expect("unable to get data directory");

        data.push(env!("CARGO_PKG_NAME"));

        fs::create_dir_all(&data)?;

        Ok(data)
    }

    fn get_installations_dir(mut data_dir: PathBuf) -> Result<PathBuf, io::Error> {
        data_dir.push("installations");

        fs::create_dir_all(&data_dir)?;

        Ok(data_dir)
    }

    fn default_log_path(data_dir: &Path) -> PathBuf {
        // TODO: rotate by count or date or something
        data_dir.join(format!("{}.log", env!("CARGO_PKG_NAME")))
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub update_interval: u64,
    pub verbose: u32,
    #[cfg(feature = "geolocation")]
    pub geo_provider: reqwest::Url,
    pub offline: bool,
    pub unchecked_downloads: bool,

    pub dirs: AppDirs,
}

impl AppConfig {
    pub fn new() -> io::Result<Self> {
        let CliArgs {
            log_file,
            update_interval,
            verbose,
            #[cfg(feature = "geolocation")]
            geo_provider,
            offline,
            unchecked_downloads,
        } = CliArgs::parse();

        Ok(Self {
            dirs: AppDirs::new(log_file)?,

            update_interval,
            verbose,
            #[cfg(feature = "geolocation")]
            geo_provider,
            offline,
            unchecked_downloads,
        })
    }
}
