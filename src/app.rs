use std::io;
use std::{collections::HashMap, sync::Arc};

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

use crate::config::AppConfig;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{tabs::TabView, AppView, ViewType};
use crate::waitable_mutex::WaitableMutex;

pub enum AppAction {
    Accepted,
    Exit,
}

pub struct App {
    views: HashMap<ViewType, Box<dyn AppView>>,
    view_stack: Vec<ViewType>,

    pub state: Arc<AppState>,
    pub stopped: bool,
    pub(crate) stop_lock: Arc<WaitableMutex<bool>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let mut instance = Self {
            state: Arc::new(AppState::new(config)),
            views: HashMap::new(),

            view_stack: vec![ViewType::Tab],
            stopped: false,
            stop_lock: Arc::new(WaitableMutex::new(false)),
        };

        instance.register_view(ViewType::Tab, Box::new(TabView::new()));

        instance
    }

    fn register_view(&mut self, tp: ViewType, view: Box<dyn AppView>) {
        self.views.insert(tp, view);
    }

    pub fn spawn_threads(&self) -> Vec<std::thread::JoinHandle<()>> {
        vec![self.state.servers.spawn_server_fetch_thread(
            self.state.locations.clone(),
            self.state.servers.clone(),
            self.stop_lock.clone(),
        )]
    }

    pub fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let size = f.size();

        // map (canvas specifically) panics with overflow if terminal size is 0
        // terminal of size 0 could happen on weird setups
        if size.area() == 0 {
            return;
        }

        if let Some(tp) = self.view_stack.last() {
            if let Some(widget) = self.views.get_mut(tp) {
                widget.draw(f, size, &self.state);
            }
        }
    }

    pub(crate) fn on_input(&mut self, event: &Event) {
        let input = match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('c' | 'C'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    self.stop();
                    None
                }
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => Some(UserInput::Left),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => Some(UserInput::Right),
                KeyEvent {
                    code: KeyCode::Up, ..
                } => Some(UserInput::Up),
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => Some(UserInput::Down),
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => Some(UserInput::Top),
                KeyEvent {
                    code: KeyCode::End, ..
                } => Some(UserInput::Bottom),
                KeyEvent {
                    code: KeyCode::Esc | KeyCode::Backspace,
                    ..
                } => Some(UserInput::Back),
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => Some(UserInput::Enter),
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => Some(UserInput::Tab),
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => Some(UserInput::Char(c)),
                _ => None,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => Some(UserInput::Up),
                MouseEventKind::ScrollDown => Some(UserInput::Down),
                _ => None,
            },
            _ => None,
        };

        if let Some(input) = input {
            for tp in self.view_stack.iter_mut().rev() {
                if let Some(widget) = self.views.get_mut(tp) {
                    if let Some(action) = widget.on_input(&input, &self.state) {
                        match action {
                            AppAction::Accepted => {
                                break;
                            }
                            AppAction::Exit => {
                                self.stop();
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.stop_lock.set(true);
    }
}
