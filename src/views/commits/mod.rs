mod draw;
mod state;

use std::sync::Arc;

use anyhow::Context;

use tokio::sync::RwLock;

use tui::widgets::ListState;

use crate::app::AppAction;
use crate::constants::GITHUB_REPO_COMMIT_ENDPOINT_URL;
use crate::datatypes::commit::{Commit, CommitsJson};
use crate::datatypes::hotkey::HotKey;
use crate::input::UserInput;
use crate::states::app::TaskResult;
use crate::states::{AppState, StatelessList};
use crate::views::{HotKeys, Input, Name};

use state::State;

pub struct Commits {
    // TODO:
    //   - on 1st launch: fetch N latest commits, save latest hash
    //   - on 2nd launch: read latest hash and fetch newer commits
    loaded: bool,

    selection: StatelessList<ListState>,
    state: Arc<RwLock<State>>,
}

impl Commits {
    pub fn new() -> Self {
        Self {
            loaded: false,
            selection: StatelessList::new(ListState::default(), false),
            state: Arc::new(RwLock::new(State::new())),
        }
    }

    pub async fn count(&self) -> usize {
        self.state.read().await.items.len()
    }

    pub async fn load(&self, app: Arc<AppState>) -> TaskResult {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );

        let commit_range = app
            .client
            .get(GITHUB_REPO_COMMIT_ENDPOINT_URL)
            .headers(headers)
            .send()
            .await
            .with_context(|| "sending commits request")?
            .error_for_status()?
            .json::<CommitsJson>()
            .await
            .with_context(|| "parsing commits response")?;

        self.update(commit_range);

        Ok(())
    }

    pub async fn update(&self, data: CommitsJson) {
        self.state
            .write()
            .await
            .items
            .append(&mut data.0.iter().map(Commit::from).collect());
    }
}

#[async_trait::async_trait]
impl Name for Commits {
    fn name(&self) -> String {
        "Recent Commit List".to_owned()
    }
}

impl HotKeys for Commits {
    fn hotkeys(&self) -> Vec<HotKey> {
        self.selection.hotkeys()
    }
}

#[async_trait::async_trait]
impl Input for Commits {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        self.selection
            .on_input(input, self.state.read().await.count())
    }
}