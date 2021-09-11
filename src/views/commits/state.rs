use std::sync::Arc;

use anyhow::Context;

use tui::widgets::ListState;

use crate::constants::GITHUB_REPO_COMMIT_ENDPOINT_URL;
use crate::datatypes::commit::{Commit, CommitsJson};
use crate::states::app::{AppState, TaskResult};
use crate::states::StatelessList;

pub struct State {
    pub items: Vec<Commit>,
    pub selection: StatelessList<ListState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selection: StatelessList::new(ListState::default(), false),
        }
    }

    pub async fn load(app: Arc<AppState>) -> TaskResult {
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

        app.commits
            .state
            .write()
            .items
            .append(&mut commit_range.0.iter().map(Commit::from).collect());

        Ok(())
    }
}
