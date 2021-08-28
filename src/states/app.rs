use std::sync::Arc;

use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub struct AppState {
    pub commits: Arc<CommitState>,
    pub installations: Arc<InstallationsState>,
    pub locations: Arc<LocationsState>,
    pub servers: Arc<ServersState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            commits: Arc::new(CommitState::new()),
            installations: Arc::new(InstallationsState::new()),
            locations: Arc::new(LocationsState::new()),
            servers: Arc::new(ServersState::new()),
        }
    }
}
