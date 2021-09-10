use std::env;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_REPOSITORY"));

#[cfg(feature = "geolocation")]
pub const DEFAULT_GEO_PROVIDER_URL: &str = "https://ifconfig.based.computer";
pub const DEFAULT_CDN_DOMAIN: &str = "unitystationfile.b-cdn.net";

pub const SERVER_LIST_URL: &str = "https://api.unitystation.org/serverlist";
pub const GITHUB_REPO_COMMIT_ENDPOINT_URL: &str =
    "https://api.github.com/repos/unitystation/unitystation/commits";
