pub mod commits;
pub mod installations;
pub mod servers;
pub mod tabs;

use std::io;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;
use tui::widgets::{ListState, TableState};

use crate::input::UserInput;
use crate::states::AppState;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
    Servers,
    Installations,
    Commits,
}

pub trait Drawable {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) -> Option<Rect>;
}

pub enum ActionResult {
    Continue,
    Stop,
    ReplaceView(ViewType),
    Exit,
}

pub trait AppView: Drawable {
    fn view_type(&self) -> ViewType;

    fn on_input(&mut self, _: &UserInput, _: &AppState) -> ActionResult {
        ActionResult::Continue
    }
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self, looped: bool) {
        if self.items.is_empty() {
            self.state.select(None);
        } else if let Some(i) = match self.state.selected() {
            None => Some(0),
            Some(i) => {
                if i < self.items.len() - 1 {
                    Some(i + 1)
                } else if looped {
                    Some(0)
                } else {
                    None
                }
            }
        } {
            self.state.select(Some(i))
        }
    }

    pub fn previous(&mut self, looped: bool) {
        if self.items.is_empty() {
            self.state.select(None);
        } else if let Some(i) = match self.state.selected() {
            None => Some(0),
            Some(i) => {
                if i != 0 {
                    Some(i - 1)
                } else if looped {
                    Some(self.items.len() - 1)
                } else {
                    None
                }
            }
        } {
            self.state.select(Some(i))
        }
    }

    pub fn select_index(&mut self, index: usize) {
        self.state.select(Some(index))
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

pub struct StatelessList {
    pub state: TableState,
}

impl StatelessList {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
        }
    }

    pub fn next(&mut self, item_count: usize) {
        if item_count == 0 {
            self.state.select(None);
        } else {
            match self.selected() {
                None => self.state.select(Some(0)),
                Some(i) => {
                    if i < item_count - 1 {
                        self.state.select(Some(i + 1))
                    }
                }
            }
        }
    }

    pub fn previous(&mut self, item_count: usize) {
        if item_count == 0 {
            self.state.select(None);
        } else {
            match self.state.selected() {
                None => self.state.select(Some(0)),
                Some(i) => {
                    if i != 0 {
                        self.state.select(Some(i - 1))
                    }
                }
            }
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}
