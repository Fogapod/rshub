use std::io;

use std::{collections::HashMap, sync::Arc};

use parking_lot::{Condvar, Mutex, RwLock};
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;
use tui::widgets::ListState;

use crate::input::UserInput;
use crate::types::Server;

use crate::views::{
    commits::CommitView, installations::InstallationView, servers::ServerView, tabs::TabView,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
    Server,
    Installations,
    Commits,
}

pub trait Drawable {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &mut AppState,
    ) -> Option<Rect>;
}

pub enum ActionResult {
    Continue,
    Stop,
    ReplaceView(ViewType),
}

pub trait AppView: Drawable {
    fn view_type(&self) -> ViewType;

    fn on_input(&mut self, _: &UserInput, _: &AppState) -> ActionResult {
        ActionResult::Continue
    }
}

type StopLock = Arc<(Mutex<bool>, Condvar)>;

pub struct AppState {
    pub servers: Arc<RwLock<HashMap<String, Server>>>,
    pub(crate) stop_lock: StopLock,
}

impl AppState {
    pub fn new(servers: Arc<RwLock<HashMap<String, Server>>>) -> Self {
        Self {
            stop_lock: Arc::new((Mutex::new(false), Condvar::new())),
            servers,
        }
    }
}

pub struct App {
    views: HashMap<ViewType, Box<dyn AppView>>,

    view_stack: Vec<ViewType>,

    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        let servers = Arc::new(RwLock::new(HashMap::new()));

        let mut instance = Self {
            state: AppState::new(servers.clone()),
            views: HashMap::new(),

            view_stack: vec![ViewType::Tab, ViewType::Server],
        };

        instance.register_view(Box::new(TabView::new()));
        instance.register_view(Box::new(ServerView::new(servers)));
        instance.register_view(Box::new(InstallationView::new()));
        instance.register_view(Box::new(CommitView::new()));

        instance
    }

    fn register_view(&mut self, view: Box<dyn AppView>) {
        self.views.insert(view.view_type(), view);
    }

    pub fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let mut area = f.size();

        for tp in self.view_stack.iter_mut() {
            if let Some(widget) = self.views.get_mut(tp) {
                match widget.draw(f, area, &mut self.state) {
                    None => break,
                    Some(ar) => area = ar,
                }
            }
        }

        // for widget in self.state.widget_stack.iter().rev() {
        //     widget.draw(f, self);
        // }
    }

    pub(crate) fn on_input(&mut self, input: &UserInput) {
        let mut view_to_insert = None;

        for tp in self.view_stack.iter_mut().rev() {
            if let Some(widget) = self.views.get_mut(tp) {
                if let ActionResult::ReplaceView(view) = widget.on_input(input, &self.state) {
                    view_to_insert = Some(view);
                }
            }
        }

        if let Some(view) = view_to_insert {
            if let Some(v) = self.view_stack.last_mut() {
                *v = view
            }
        }
    }
}
