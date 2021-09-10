use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use anyhow::Result;

use crate::app::AppAction;
use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::states::events::EventsState;
use crate::states::help::HelpState;
use crate::states::help::HotKey;
#[cfg(feature = "geolocation")]
use crate::states::LocationsState;
use crate::states::{CommitState, ServersState, VersionsState};

pub type TaskResult = Result<()>;

pub struct AppState {
    pub config: AppConfig,
    pub commits: Arc<RwLock<CommitState>>,
    pub versions: Arc<RwLock<VersionsState>>,
    #[cfg(feature = "geolocation")]
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Arc<RwLock<ServersState>>,
    pub events: Arc<RwLock<EventsState>>,

    pub help: Mutex<HelpState>,

    pub client: reqwest::Client,

    panic_bool: Arc<AtomicBool>,
}

impl AppState {
    pub async fn new(config: AppConfig, panic_bool: Arc<AtomicBool>) -> Arc<Self> {
        #[cfg(feature = "geolocation")]
        let locations = Arc::new(RwLock::new(LocationsState::new(&config).await));
        let versions = Arc::new(RwLock::new(VersionsState::new(&config).await));
        let servers = Arc::new(RwLock::new(ServersState::new(&config).await));
        let events = Arc::new(RwLock::new(EventsState::new(&config).await));

        let instance = Arc::new(Self {
            commits: Arc::new(RwLock::new(CommitState::new().await)),
            versions: versions.clone(),
            #[cfg(feature = "geolocation")]
            locations: locations.clone(),
            servers: servers.clone(),
            events: events.clone(),
            config,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("creating client"),

            help: Mutex::new(HelpState::new()),

            panic_bool,
        });

        events.write().await.run(instance.clone()).await;
        servers.write().await.run(instance.clone()).await;
        #[cfg(feature = "geolocation")]
        locations.write().await.run(instance.clone()).await;
        versions.write().await.run(instance.clone()).await;

        instance
    }

    pub async fn on_action(&self, action: &AppAction, app: Arc<AppState>) {
        log::debug!("action: {:?}", &action);

        let f = match action {
            AppAction::ConnectToServer { version, address } => Some(tokio::spawn(
                VersionsState::launch(Arc::clone(&app), version.clone(), Some(address.clone())),
            )),
            AppAction::InstallVersion(version) => Some(tokio::spawn(VersionsState::install(
                Arc::clone(&app),
                version.clone(),
            ))),
            AppAction::LaunchVersion(version) => Some(tokio::spawn(VersionsState::launch(
                Arc::clone(&app),
                version.clone(),
                None,
            ))),
            AppAction::AbortVersionInstallation(version) => Some(tokio::spawn(
                VersionsState::abort_installation(Arc::clone(&app), version.clone()),
            )),
            AppAction::UninstallVersion(version) => Some(tokio::spawn(VersionsState::uninstall(
                Arc::clone(&app),
                version.clone(),
            ))),

            _ => None,
        };

        if let Some(f) = f {
            self.watch_task(f).await;
        }
    }

    pub fn display_help(&self, view_name: &str, keys: &[HotKey]) {
        let mut help = self.help.lock().unwrap();
        help.set_name(view_name);
        help.set_hotkeys(keys);
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
