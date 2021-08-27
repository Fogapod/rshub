use std::io;

use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crossterm::event::{Event, KeyCode, MouseEventKind};

use crate::input::UserInput;
use crate::states::{AppState, ServersState};

use crate::views::{
    commits::CommitView, installations::InstallationView, servers::ServerView, tabs::TabView,
    ActionResult, AppView, ViewType,
};

use crate::waitable_mutex::WaitableMutex;

pub struct App {
    views: HashMap<ViewType, Box<dyn AppView>>,

    view_stack: Vec<ViewType>,

    pub state: Arc<AppState>,
    pub stopped: bool,
    pub(crate) stop_lock: Arc<WaitableMutex<bool>>,
}

impl App {
    pub fn new() -> Self {
        let mut instance = Self {
            state: Arc::new(AppState::new()),
            views: HashMap::new(),

            view_stack: vec![ViewType::Tab, ViewType::Servers],
            stopped: false,
            stop_lock: Arc::new(WaitableMutex::new(false)),
        };

        instance.register_view(Box::new(TabView::new()));
        instance.register_view(Box::new(ServerView::new()));
        instance.register_view(Box::new(InstallationView::new()));
        instance.register_view(Box::new(CommitView::new()));

        instance
    }

    fn register_view(&mut self, view: Box<dyn AppView>) {
        self.views.insert(view.view_type(), view);
    }

    pub fn spawn_threads(&self) -> Vec<std::thread::JoinHandle<()>> {
        vec![ServersState::spawn_server_fetch_thread(
            Duration::from_secs(20),
            self.state.clone(),
            self.stop_lock.clone(),
        )]
    }

    pub fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let mut area = f.size();

        for tp in self.view_stack.iter_mut() {
            if let Some(widget) = self.views.get_mut(tp) {
                match widget.draw(f, area, &self.state) {
                    None => break,
                    Some(ar) => area = ar,
                }
            }
        }

        // for widget in self.state.widget_stack.iter().rev() {
        //     widget.draw(f, self);
        // }
    }

    pub(crate) fn on_input(&mut self, event: &Event) {
        let input = match event {
            Event::Key(key) => match key.code {
                KeyCode::Left => Some(UserInput::Left),
                KeyCode::Right => Some(UserInput::Right),
                KeyCode::Up => Some(UserInput::Up),
                KeyCode::Down => Some(UserInput::Down),
                KeyCode::Esc | KeyCode::Backspace => Some(UserInput::Back),
                KeyCode::Enter => Some(UserInput::Enter),
                KeyCode::Tab => Some(UserInput::Tab),
                KeyCode::Char(c) => Some(UserInput::Char(c)),
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
            let mut actions = Vec::new();

            for tp in self.view_stack.iter_mut().rev() {
                if let Some(widget) = self.views.get_mut(tp) {
                    match widget.on_input(&input, &self.state) {
                        ActionResult::Stop => {
                            break;
                        }
                        ActionResult::Exit => {
                            actions.push(ActionResult::Exit);
                            break;
                        }
                        result => actions.push(result),
                    }
                }
            }

            for action in actions {
                match action {
                    ActionResult::Exit => self.stop(),
                    ActionResult::ReplaceView(view) => {
                        if let Some(v) = self.view_stack.last_mut() {
                            *v = view
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.stop_lock.set(true);
    }
}
