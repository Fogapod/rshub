use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<RwLock<CommitState>>,
    pub installations: Arc<RwLock<InstallationsState>>,
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Arc<RwLock<ServersState>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let locations = LocationsState::new(&config).await;
        let installations = InstallationsState::new().await;
        let servers = ServersState::new(&config, locations.clone(), installations.clone()).await;

        Self {
            commits: CommitState::new().await,
            installations,
            locations,
            servers,
            config,
        }
    }
}
