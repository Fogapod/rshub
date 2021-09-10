mod draw;
mod hotkeys;
mod input;
mod state;

use std::sync::Arc;

use crossterm::event::KeyCode;

use tokio::sync::RwLock;

use tui::widgets::TableState;

use crate::states::StatelessList;
use crate::views::Name;

use state::State;

pub struct Versions {
    state: Arc<RwLock<State>>,
    selection: StatelessList<TableState>,
}

impl Versions {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(State::new())),
            selection: StatelessList::new(TableState::default(), false),
        }
    }
}

impl Name for Versions {
    fn name(&self) -> String {
        "Version List".to_owned()
    }
}
