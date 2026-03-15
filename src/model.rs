use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The source a PR record was fetched from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchSource {
    GhCli,
    GitHubApi,
}

/// A normalized pull request record, independent of fetch source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub repo: String,
    pub url: String,
    pub updated_at: DateTime<Utc>,
    pub draft: bool,
}

/// Outcome of fetching PRs for one repository.
#[derive(Debug)]
#[allow(dead_code)] // `repo` and `source` are used in tests and future UI features
pub struct FetchResult {
    pub repo: String,
    pub prs: Vec<PullRequest>,
    pub source: FetchSource,
    pub warning: Option<String>,
}
