use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use anyhow::Result;

use crate::app::{AppAction, StopSignal};
use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::hotkey::HotKey;
use crate::states::events::EventsState;
use crate::states::help::HelpState;
#[cfg(feature = "geolocation")]
use crate::states::LocationsState;
use crate::views::commits::Commits;
use crate::views::servers::Servers;
use crate::views::versions::Versions;

pub type TaskResult = Result<()>;

pub struct AppState {
    pub config: AppConfig,
    pub commits: Commits,
    pub versions: Versions,
    #[cfg(feature = "geolocation")]
    pub locations: Arc<RwLock<LocationsState>>,
    pub servers: Servers,
    pub events: Arc<RwLock<EventsState>>,

    pub help: Mutex<HelpState>,

    pub client: reqwest::Client,

    kill_switch: mpsc::Sender<StopSignal>,
}

impl AppState {
    pub fn new(config: AppConfig, kill_switch: mpsc::Sender<StopSignal>) -> Self {
        #[cfg(feature = "geolocation")]
        let locations = Arc::new(RwLock::new(LocationsState::new(&config)));
        let versions = Versions::new();
        let servers = Servers::new();
        let events = Arc::new(RwLock::new(EventsState::new(&config)));

        Self {
            commits: Commits::new(),
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

            kill_switch,
        }
    }

    pub async fn run(&self, instance: Arc<Self>) {
        self.events.write().run(instance.clone()).await;
        self.servers.run(instance.clone()).await;
        #[cfg(feature = "geolocation")]
        self.locations.write().run(instance.clone()).await;
        //self.versions.write().await.run(instance.clone()).await;
    }

    pub async fn on_action(&self, action: &AppAction, app: Arc<AppState>) {
        log::debug!("action: {:?}", &action);

        // let f = match action {
        //     AppAction::ConnectToServer { version, address } => Some(tokio::spawn(
        //         VersionsState::launch(Arc::clone(&app), version.clone(), Some(address.clone())),
        //     )),
        //     AppAction::InstallVersion(version) => Some(tokio::spawn(VersionsState::install(
        //         Arc::clone(&app),
        //         version.clone(),
        //     ))),
        //     AppAction::LaunchVersion(version) => Some(tokio::spawn(VersionsState::launch(
        //         Arc::clone(&app),
        //         version.clone(),
        //         None,
        //     ))),
        //     AppAction::AbortVersionInstallation(version) => Some(tokio::spawn(
        //         VersionsState::abort_installation(Arc::clone(&app), version.clone()),
        //     )),
        //     AppAction::UninstallVersion(version) => Some(tokio::spawn(VersionsState::uninstall(
        //         Arc::clone(&app),
        //         version.clone(),
        //     ))),

        //     _ => None,
        // };

        // if let Some(f) = f {
        //     self.watch_task(f).await;
        // }
    }

    pub fn display_help(&self, view_name: &str, keys: &[HotKey]) {
        let mut help = self.help.lock();
        help.set_name(view_name);
        help.set_hotkeys(keys);
    }

    pub async fn watch_task(&self, task: JoinHandle<TaskResult>) {
        tokio::spawn(Self::wrap_task(
            task,
            self.kill_switch.clone(),
            self.events.clone(),
        ));
    }

    async fn wrap_task(
        task: JoinHandle<TaskResult>,
        kill_switch: mpsc::Sender<StopSignal>,
        events: Arc<RwLock<EventsState>>,
    ) {
        match task.await {
            Err(err) => {
                log::warn!("join error: {:?}", &err);

                if err.is_panic() {
                    log::error!("error is panic, setting panic to exit on next tick");
                    kill_switch.send(StopSignal::Panic).await.unwrap();
                }
            }
            Ok(result) => {
                if let Err(err) = result {
                    events.read().error(err).await;
                }
            }
        }
    }
}
