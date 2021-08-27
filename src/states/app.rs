use std::sync::Arc;

use crate::states::{CommitState, LocationsState, ServersState};

pub struct AppState {
    pub servers: Arc<ServersState>,
    pub locations: Arc<LocationsState>,
    pub commits: Arc<CommitState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(ServersState::new()),
            locations: Arc::new(LocationsState::new()),
            commits: Arc::new(CommitState::new()),
        }
    }
}
