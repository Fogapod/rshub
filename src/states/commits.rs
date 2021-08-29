use tokio::sync::RwLock;

use crate::constants::USER_AGENT;
use crate::datatypes::commit::{Commit, CommitRange};

const GITHUB_REPO_URL: &str = "https://api.github.com/repos/unitystation/unitystation/commits";

pub struct CommitState {
    pub items: RwLock<Vec<Commit>>,
    client: reqwest::Client,
}

impl CommitState {
    pub async fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );

        Self {
            items: RwLock::new(Vec::new()),
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(headers)
                .build()
                .expect("creating client"),
        }
    }

    pub async fn count(&self) -> usize {
        self.items.read().await.len()
    }

    pub async fn load(&self) {
        let req = match self.client.get(GITHUB_REPO_URL).send().await {
            Ok(req) => req,
            Err(err) => {
                log::error!("error creating request: {}", err);
                return;
            }
        };
        let req = match req.error_for_status() {
            Ok(req) => req,
            Err(err) => {
                log::error!("bad status: {}", err);
                return;
            }
        };
        let resp = match req.json::<CommitRange>().await {
            Ok(resp) => resp,
            Err(err) => {
                log::error!("error decoding request: {}", err);
                return;
            }
        };
        if let Err(e) = self.update(resp).await {
            log::error!("error updating commits: {}", e);
        }
    }

    pub async fn update(&self, data: CommitRange) -> Result<(), Box<dyn std::error::Error>> {
        let mut commits = self.items.write().await;

        commits.append(&mut data.0.iter().map(Commit::from).collect());

        Ok(())
    }
}
