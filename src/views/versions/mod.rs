mod draw;
mod hotkeys;
mod input;
mod state;

use std::sync::Arc;

use parking_lot::RwLock;

use crate::views::Name;

use state::State;

#[derive(Clone)]
pub struct Versions {
    pub state: Arc<RwLock<State>>,
}

impl Versions {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(State::new())),
        }
    }

    pub fn count(&self) -> usize {
        self.state.read().items.len()
    }
}

impl Name for Versions {
    fn name(&self) -> String {
        "Version List".to_owned()
    }
}
