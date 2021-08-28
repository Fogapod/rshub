use std::sync::Arc;

use crate::config::AppConfig;
use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<CommitState>,
    pub installations: Arc<InstallationsState>,
    pub locations: Arc<LocationsState>,
    pub servers: Arc<ServersState>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            commits: Arc::new(CommitState::new()),
            installations: Arc::new(InstallationsState::new()),
            locations: Arc::new(LocationsState::new(&config)),
            servers: Arc::new(ServersState::new(&config)),
            config,
        }
    }
}
