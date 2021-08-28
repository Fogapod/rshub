use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct CommitAuthor {
    pub name: String,
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct CommitData {
    pub author: CommitAuthor,
    pub message: String,
    // pub added: usize,
    // pub removed: usize,
}

#[derive(Deserialize, Debug)]
pub struct GitHubJunkCommit {
    pub sha: String,
    pub commit: CommitData,
}

#[derive(Deserialize, Debug)]
pub struct CommitRange(pub Vec<GitHubJunkCommit>);

#[derive(Debug)]
pub struct Commit {
    pub sha: String,
    pub title: String,
    pub message: String,
    pub date: String,
    // pub added: usize,
    // pub removed: usize,
    pub author: CommitAuthor,
}

impl From<&GitHubJunkCommit> for Commit {
    fn from(commit: &GitHubJunkCommit) -> Self {
        Self {
            sha: commit.sha.to_owned(),
            date: commit.commit.author.date.clone(),
            author: commit.commit.author.clone(),
            title: commit.commit.message.lines().next().unwrap().to_owned(),
            message: commit.commit.message.clone(),
        }
    }
}
