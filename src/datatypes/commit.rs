use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct CommitAuthorJson {
    pub name: String,
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct CommitJson {
    pub author: CommitAuthorJson,
    pub message: String,
    // pub added: usize,
    // pub removed: usize,
}

#[derive(Deserialize, Debug)]
pub struct GitHubJunkCommitJson {
    pub sha: String,
    pub commit: CommitJson,
}

#[derive(Deserialize, Debug)]
pub struct CommitsJson(pub Vec<GitHubJunkCommitJson>);

#[derive(Debug)]
pub struct Commit {
    pub sha: String,
    pub title: String,
    pub message: String,
    pub date: String,
    // pub added: usize,
    // pub removed: usize,
    pub author: CommitAuthorJson,
}

impl From<&GitHubJunkCommitJson> for Commit {
    fn from(commit: &GitHubJunkCommitJson) -> Self {
        Self {
            sha: commit.sha.to_owned(),
            date: commit.commit.author.date.clone(),
            author: commit.commit.author.clone(),
            title: commit.commit.message.lines().next().unwrap().to_owned(),
            message: commit.commit.message.clone(),
        }
    }
}
