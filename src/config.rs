use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = clap::crate_version!(), about = "UnityStation server hub")]
pub struct AppArgs {
    /// Log file path
    #[clap(short, long)]
    pub log_file: Option<PathBuf>,
    /// Server list update interval, in seconds
    #[clap(short, long, default_value = "20")]
    pub update_interval: u64,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,
    /// Geolocation provider (ifconfig.co compatible)
    #[clap(long, default_value = "https://ifconfig.based.computer")]
    pub geo_provider: String,
}

#[derive(Debug)]
pub struct AppConfig {
    pub args: AppArgs,
    pub data_dir: PathBuf,
}

impl AppConfig {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            args: AppArgs::parse(),
            data_dir: Self::get_data_dir()?,
        })
    }

    fn get_data_dir() -> Result<PathBuf, io::Error> {
        let mut data = dirs_next::data_dir().expect("unable to get data directory");

        data.push(env!("CARGO_PKG_NAME"));

        fs::create_dir_all(&data)?;

        Ok(data)
    }
}
