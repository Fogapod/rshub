use std::sync::Arc;

use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::geolocation::IP;
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
        let (tx, rx) = mpsc::unbounded_channel::<IP>();

        let servers = Arc::new(ServersState::new(&config).await);
        let locations = Arc::new(LocationsState::new(&config, tx).await);

        tokio::task::spawn(LocationsState::location_fetch_task(locations.clone(), rx));
        tokio::task::spawn(ServersState::server_fetch_task(
            servers.clone(),
            locations.clone(),
        ));

        Self {
            commits: Arc::new(CommitState::new().await),
            installations: Arc::new(InstallationsState::new().await),
            locations,
            servers,
            config,
        }
    }
}
