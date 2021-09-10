use std::sync::Arc;

use anyhow::Context;

use crate::constants::GITHUB_REPO_COMMIT_ENDPOINT_URL;
use crate::datatypes::commit::{Commit, CommitsJson};
use crate::states::app::{AppState, TaskResult};

pub struct CommitState {
    pub items: Vec<Commit>,
}

impl CommitState {
    pub async fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn count(&self) -> usize {
        self.items.len()
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

        app.commits.write().await.update(commit_range);

        Ok(())
    }

    pub fn update(&mut self, data: CommitsJson) {
        self.items
            .append(&mut data.0.iter().map(Commit::from).collect());
    }
}
