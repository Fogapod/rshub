use std::env;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_REPOSITORY"));

pub const DEFAULT_GEO_PROVIDER_URL: &str = "https://ifconfig.based.computer";

pub const SERVER_LIST_URL: &str = "https://api.unitystation.org/serverlist";
