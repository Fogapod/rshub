use std::sync::Arc;

use anyhow::Context;

use tokio::sync::RwLock;

use crate::datatypes::commit::{Commit, CommitRange};
use crate::states::app::TaskResult;

const GITHUB_REPO_URL: &str = "https://api.github.com/repos/unitystation/unitystation/commits";

pub struct CommitState {
    pub items: Vec<Commit>,
    client: reqwest::Client,
}

impl CommitState {
    pub async fn new(client: reqwest::Client) -> Self {
        Self {
            items: Vec::new(),
            client,
        }
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn load(commits: Arc<RwLock<Self>>) -> TaskResult {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );

        let req = commits
            .read()
            .await
            .client
            .get(GITHUB_REPO_URL)
            .headers(headers)
            .send()
            .await
            .with_context(|| "Creating commits request")?;

        let req = req
            .error_for_status()
            .with_context(|| "Reading commits request")?;

        let resp = req
            .json::<CommitRange>()
            .await
            .with_context(|| "Decoding commits request")?;

        commits.write().await.update(resp);

        Ok(())
    }

    pub fn update(&mut self, data: CommitRange) {
        self.items
            .append(&mut data.0.iter().map(Commit::from).collect());
    }
}
