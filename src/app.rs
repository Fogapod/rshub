use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use tui::backend::CrosstermBackend;
use tui::terminal::{Frame, Terminal};

use crossterm::event::EventStream;

use futures::StreamExt;

use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::datatypes::game_version::GameVersion;
use crate::datatypes::server::Address;
use crate::input::UserInput;
use crate::states::app::AppState;
#[cfg(feature = "geolocation")]
use crate::views::world::World;
use crate::views::{events::EventsView, help::Help, tabs::Tabs, AppView, Draw, ViewType};

#[derive(Debug)]
pub enum StopSignal {
    UserExit,
    Panic,
}

#[derive(Debug)]
pub enum AppAction {
    // view management
    OpenView(ViewType),
    CloseView,
    // installations
    InstallVersion(GameVersion),
    AbortVersionInstallation(GameVersion),
    UninstallVersion(GameVersion),
    LaunchVersion(GameVersion),
    ConnectToServer {
        version: GameVersion,
        address: Address,
    },
}

pub struct App {
    pub state: Arc<AppState>,

    views: HashMap<ViewType, Box<dyn AppView>>,
    view_stack: Vec<ViewType>,

    events_view: EventsView,

    pub kill_switch: mpsc::Sender<StopSignal>,
    kill_switch_recv: mpsc::Receiver<StopSignal>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let (kill_switch, kill_switch_recv) = mpsc::channel(1);
        let state = Arc::new(AppState::new(config, kill_switch.clone()));

        let mut instance = Self {
            state,

            views: HashMap::new(),
            view_stack: vec![ViewType::Tab],

            events_view: EventsView {},

            kill_switch,
            kill_switch_recv,
        };

        instance.register_view(ViewType::Tab, Box::new(Tabs::new()));
        #[cfg(feature = "geolocation")]
        instance.register_view(ViewType::World, Box::new(World {}));
        instance.register_view(ViewType::Help, Box::new(Help {}));

        instance
    }

    fn register_view(&mut self, tp: ViewType, view: Box<dyn AppView>) {
        self.views.insert(tp, view);
    }

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        self.state.run(Arc::clone(&self.state)).await;

        let interval = Duration::from_millis(200);
        let mut delay = tokio::time::interval(interval);
        let mut reader = EventStream::new();

        loop {
            tokio::select! {
                _ = delay.tick() => {
                    terminal.draw(|f| self.draw(f)).unwrap();
                },
                maybe_event = reader.next() => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Ok(valid_input) = UserInput::try_from(&event) {
                                self.on_input(&valid_input).await;
                            }
                        },
                        Some(Err(err)) => {
                            log::error!("Error reading input: {}", err);
                            break;
                        }
                        None => {
                            log::error!("Input channel closed somehow");
                            break
                        },
                    }
                },
                stop = self.kill_switch_recv.recv() => {
                    if let Some(stop) = stop {
                        match stop {
                            StopSignal::UserExit => {
                                log::info!("app stopped, cleaning up");
                                break;
                            }
                            StopSignal::Panic => {
                                log::error!("app panicked, cleaning up");
                                break;
                            }
                        }
                    } else {
                        log::error!("kill_switch channel closed somehow");
                        break;
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>) {
        if let Some(tp) = self.view_stack.last() {
            if let Some(widget) = self.views.get_mut(tp) {
                use tui::layout::{Constraint, Direction, Layout};

                let area = if self.state.events.read().current_event.is_some() {
                    let chunks = Layout::default()
                        .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
                        .direction(Direction::Vertical)
                        .split(f.size());

                    self.events_view.draw(f, chunks[1], Arc::clone(&self.state));

                    chunks[0]
                } else {
                    f.size()
                };

                widget.draw(f, area, Arc::clone(&self.state));
            }
        }
    }

    pub(crate) async fn on_input(&mut self, input: &UserInput) {
        log::debug!("input: {:?}", input);

        match input {
            UserInput::Quit => {
                self.kill_switch.send(StopSignal::UserExit).await.unwrap();
            }
            UserInput::Help => {
                if let Some(top_view_type) = self.view_stack.last() {
                    if top_view_type == &ViewType::Help {
                        return;
                    }
                    if let Some(top_view) = self.views.get(top_view_type) {
                        self.view_stack.push(ViewType::Help);
                        self.state
                            .display_help(&top_view.name(), &top_view.hotkeys());
                    }
                }
            }
            _ => {
                if let Some(top_widget_type) = self.view_stack.last() {
                    if let Some(widget) = self.views.get_mut(top_widget_type) {
                        if let Some(action) = widget.on_input(input, self.state.clone()).await {
                            match action {
                                AppAction::OpenView(view) => {
                                    self.view_stack.push(view);
                                }
                                AppAction::CloseView => {
                                    self.view_stack.pop();
                                }

                                _ => self.state.on_action(&action, Arc::clone(&self.state)).await,
                            }
                        }
                    }
                }
            }
        }
    }
}
