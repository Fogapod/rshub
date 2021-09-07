use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use anyhow::Result;

use crate::app::AppAction;
use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::states::events::EventsState;
use crate::states::{CommitState, InstallationsState, LocationsState, ServersState};

pub type TaskResult = Result<()>;

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<RwLock<CommitState>>,
    pub installations: Arc<RwLock<InstallationsState>>,
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Arc<RwLock<ServersState>>,
    pub events: Arc<RwLock<EventsState>>,

    pub client: reqwest::Client,

    panic_bool: Arc<AtomicBool>,
}

impl AppState {
    pub async fn new(config: AppConfig, panic_bool: Arc<AtomicBool>) -> Arc<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        let locations = Arc::new(RwLock::new(LocationsState::new(&config).await));
        let installations = Arc::new(RwLock::new(InstallationsState::new(&config).await));
        let servers = Arc::new(RwLock::new(ServersState::new(&config).await));
        let events = Arc::new(RwLock::new(EventsState::new(&config).await));

        let instance = Arc::new(Self {
            commits: Arc::new(RwLock::new(CommitState::new(client.clone()).await)),
            installations: installations.clone(),
            locations: locations.clone(),
            servers: servers.clone(),
            events: events.clone(),
            config,
            client,

            panic_bool,
        });

        events.write().await.run(instance.clone()).await;
        servers.write().await.run(instance.clone()).await;
        locations.write().await.run(instance.clone()).await;
        installations.write().await.run(instance.clone()).await;

        instance
    }

    pub async fn on_action(&self, action: &AppAction, app: Arc<AppState>) {
        log::debug!("action: {:?}", &action);

        let f =
            match action {
                AppAction::ConnectToServer { version, address } => {
                    Some(tokio::spawn(InstallationsState::launch(
                        Arc::clone(&app),
                        version.clone(),
                        Some(address.clone()),
                    )))
                }
                AppAction::LaunchVersion(version) => Some(tokio::spawn(
                    InstallationsState::launch(Arc::clone(&app), version.clone(), None),
                )),
                AppAction::AbortVersionInstallation(version) => Some(tokio::spawn(
                    InstallationsState::abort_installation(Arc::clone(&app), version.clone()),
                )),
                AppAction::UninstallVersion(version) => Some(tokio::spawn(
                    InstallationsState::uninstall(Arc::clone(&app), version.clone()),
                )),
                _ => None,
            };

        if let Some(f) = f {
            self.watch_task(f).await;
        }
    }

    pub async fn watch_task(&self, task: JoinHandle<TaskResult>) {
        tokio::spawn(Self::wrap_task(
            task,
            self.panic_bool.clone(),
            self.events.clone(),
        ));
    }

    async fn wrap_task(
        task: JoinHandle<TaskResult>,
        panic_bool: Arc<AtomicBool>,
        events: Arc<RwLock<EventsState>>,
    ) {
        match task.await {
            Err(err) => {
                log::warn!("join error: {:?}", &err);

                if err.is_panic() {
                    log::error!("error is panic, setting panic to exit on next tick");
                    panic_bool.store(true, Ordering::Relaxed);
                }
            }
            Ok(result) => {
                if let Err(err) = result {
                    events.read().await.error(err).await;
                }
            }
        }
    }
}
