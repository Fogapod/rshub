mod draw;
mod state;

use std::sync::Arc;

use parking_lot::RwLock;

use crate::app::AppAction;

use crate::datatypes::hotkey::HotKey;
use crate::input::UserInput;

use crate::states::AppState;
use crate::views::{HotKeys, Input, Name};

use state::State;

#[derive(Clone)]
pub struct Commits {
    // TODO:
    //   - on 1st launch: fetch N latest commits, save latest hash
    //   - on 2nd launch: read latest hash and fetch newer commits
    loaded: bool,

    state: Arc<RwLock<State>>,
}

impl Commits {
    pub fn new() -> Self {
        Self {
            loaded: false,

            state: Arc::new(RwLock::new(State::new())),
        }
    }

    pub fn count(&self) -> usize {
        self.state.read().items.len()
    }

    pub async fn load(&self, app: Arc<AppState>) {
        app.watch_task(tokio::spawn(State::load(Arc::clone(&app))))
            .await;
    }
}

impl Name for Commits {
    fn name(&self) -> String {
        "Recent Commit List".to_owned()
    }
}

impl HotKeys for Commits {
    fn hotkeys(&self) -> Vec<HotKey> {
        self.state.read().selection.hotkeys()
    }
}

#[async_trait::async_trait]
impl Input for Commits {
    async fn on_input(&mut self, input: &UserInput, _: Arc<AppState>) -> Option<AppAction> {
        self.state.write().selection.on_input(input, self.count())
    }
}
