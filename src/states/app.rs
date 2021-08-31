use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

use crate::config::AppConfig;
use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub type TaskResult = Result<(), Box<dyn std::error::Error + Send>>;
pub type TaskQueue = Arc<RwLock<mpsc::UnboundedSender<JoinHandle<TaskResult>>>>;

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<RwLock<CommitState>>,
    pub installations: Arc<RwLock<InstallationsState>>,
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Arc<RwLock<ServersState>>,

    pub tasks: TaskQueue,
}

impl AppState {
    pub async fn new(config: AppConfig, panic_bool: Arc<AtomicBool>) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tasks = Arc::new(RwLock::new(tx));

        let locations = LocationsState::new(&config, tasks.clone()).await;
        let installations = InstallationsState::new(&config, tasks.clone()).await;
        let servers = ServersState::new(
            &config,
            tasks.clone(),
            locations.clone(),
            installations.clone(),
        )
        .await;

        tokio::spawn(Self::panic_watcher_super_task(panic_bool, rx));

        Arc::new(Self {
            commits: CommitState::new().await,
            installations,
            locations,
            servers,
            config,
            tasks,
        })
    }

    async fn panic_watcher_super_task(
        panic_bool: Arc<AtomicBool>,
        mut recv: mpsc::UnboundedReceiver<JoinHandle<TaskResult>>,
    ) {
        while let Some(task) = recv.recv().await {
            let panic_bool = panic_bool.clone();

            tokio::spawn(async move {
                if let Err(err) = task.await {
                    log::warn!("super task task error: {:?}", &err);

                    if err.is_panic() {
                        log::error!("error is panic, setting panic to exit on next tick");
                        panic_bool.store(true, Ordering::Relaxed);
                    }

                    // TODO: handle other errors
                }
            });
        }

        log::info!("task channel closed");
    }
}
