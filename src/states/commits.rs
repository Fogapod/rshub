use std::collections::HashMap;

use parking_lot::RwLock;

use crate::constants::USER_AGENT;
use crate::datatypes::commit::{Commit, CommitRange};

const GITHUB_REPO_URL: &str = "https://api.github.com/repos/unitystation/unitystation/commits";

pub struct CommitState {
    pub commits: RwLock<HashMap<String, Commit>>,
    client: reqwest::blocking::Client,
}

impl Default for CommitState {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitState {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );

        Self {
            commits: RwLock::new(HashMap::new()),
            client: reqwest::blocking::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(headers)
                .build()
                .expect("creating client"),
        }
    }

    pub fn load(&self) {
        let req = match self.client.get(GITHUB_REPO_URL).send() {
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
        let resp = match req.json::<CommitRange>() {
            Ok(resp) => resp,
            Err(err) => {
                log::error!("error decoding request: {}", err);
                return;
            }
        };
        if let Err(e) = self.update(resp) {
            log::error!("error updating commits: {}", e);
        }
    }

    pub fn update(&self, data: CommitRange) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("{:#?}", &data);

        let mut commits = self.commits.write();

        for c in data.0 {
            commits.insert(c.sha.clone(), c.into());
        }

        Ok(())
    }
}
