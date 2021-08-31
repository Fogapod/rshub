use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Clap;

use crate::constants::DEFAULT_GEO_PROVIDER_URL;

#[derive(Clap, Debug)]
#[clap(version = clap::crate_version!(), about = "UnityStation server hub")]
struct CliArgs {
    /// Log file path
    #[clap(short, long)]
    log_file: Option<PathBuf>,
    /// Server list update interval, in seconds
    #[clap(short, long, default_value = "20")]
    update_interval: u64,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: u32,
    /// Geolocation provider (ifconfig.co compatible)
    #[clap(long, default_value = DEFAULT_GEO_PROVIDER_URL)]
    geo_provider: reqwest::Url,
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
    pub geo_provider: reqwest::Url,

    pub dirs: AppDirs,
}

impl AppConfig {
    pub fn new() -> Result<Self, io::Error> {
        let args = CliArgs::parse();

        Ok(Self {
            dirs: AppDirs::new(args.log_file)?,

            update_interval: args.update_interval,
            verbose: args.verbose,
            geo_provider: args.geo_provider,
        })
    }
}
