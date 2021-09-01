use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub type TaskResult = Result<(), Box<dyn std::error::Error + Send>>;
pub type TaskQueue = Arc<RwLock<mpsc::UnboundedSender<JoinHandle<TaskResult>>>>;

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<RwLock<CommitState>>,
    pub installations: Arc<RwLock<InstallationsState>>,
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Arc<RwLock<ServersState>>,

    pub client: reqwest::Client,
    pub tasks: TaskQueue,
}

impl AppState {
    pub async fn new(config: AppConfig, panic_bool: Arc<AtomicBool>) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        let locations = Arc::new(RwLock::new(LocationsState::new(&config).await));
        let installations = Arc::new(RwLock::new(InstallationsState::new(&config).await));
        let servers = Arc::new(RwLock::new(ServersState::new(&config).await));

        tokio::spawn(Self::panic_watcher_super_task(panic_bool, rx));

        let instance = Arc::new(Self {
            commits: Arc::new(RwLock::new(CommitState::new(client.clone()).await)),
            installations: installations.clone(),
            locations: locations.clone(),
            servers: servers.clone(),
            config,
            client,
            tasks: Arc::new(RwLock::new(tx)),
        });

        servers.write().await.run(instance.clone()).await;
        locations.write().await.run(instance.clone()).await;
        installations.write().await.run(instance.clone()).await;

        instance
    }

    pub async fn watch_task(&self, task: JoinHandle<TaskResult>) {
        self.tasks
            .write()
            .await
            .send(task)
            .expect("spawn watched task");
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
