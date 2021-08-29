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
    pub async fn new(config: AppConfig) -> Self {
        let locations = LocationsState::new(&config).await;
        let servers = ServersState::new(&config, locations.clone()).await;

        Self {
            commits: Arc::new(CommitState::new().await),
            installations: InstallationsState::new().await,
            locations,
            servers,
            config,
        }
    }
}
