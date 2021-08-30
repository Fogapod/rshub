use crate::datatypes::server::GameVersion;

#[derive(Debug)]
pub enum InstallationAction {
    VersionDiscovered {
        new: GameVersion,
        old: Option<GameVersion>,
    },
    Install(GameVersion),
    Uninstall(GameVersion),
}

#[derive(Debug)]
pub enum InstallationKind {
    Discovered,
    Installed,
    // Corrupted,
    Downloading { progress: usize, total: usize },
    Unpacking,
}

#[derive(Debug)]
pub struct Installation {
    pub version: GameVersion,
    pub kind: InstallationKind,
}
