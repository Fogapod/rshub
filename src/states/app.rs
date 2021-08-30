use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::future::try_join_all;
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
    pub async fn new(config: AppConfig, panic_bool: Arc<AtomicBool>) -> Arc<Self> {
        let mut managed_tasks = Vec::new();

        let locations = LocationsState::new(&config, &mut managed_tasks).await;
        let installations = InstallationsState::new(&config, &mut managed_tasks).await;
        let servers = ServersState::new(
            &config,
            &mut managed_tasks,
            locations.clone(),
            installations.clone(),
        )
        .await;

        tokio::spawn(Self::panic_watcher_super_task(panic_bool, managed_tasks));

        Arc::new(Self {
            commits: CommitState::new().await,
            installations,
            locations,
            servers,
            config,
        })
    }

    async fn panic_watcher_super_task(
        panicked: Arc<AtomicBool>,
        tasks: Vec<tokio::task::JoinHandle<()>>,
    ) {
        if let Err(err) = try_join_all(tasks).await {
            log::error!("super task task error: {:?}", &err);

            if err.is_panic() {
                panicked.store(true, Ordering::Relaxed);
            }
        }
    }
}
