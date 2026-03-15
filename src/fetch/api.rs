use crate::model::{FetchResult, FetchSource, PullRequest};
use anyhow::{Context, Result};
use chrono::DateTime;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Deserialize;
use tracing::debug;

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Build a blocking reqwest client with auth and user-agent headers.
fn build_client(token: &str) -> Result<reqwest::blocking::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))
            .context("Invalid GitHub token characters")?,
    );
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static("pr-tracker/0.1"),
    );

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .https_only(true)
        .build()
        .context("Failed to build HTTP client")
}

/// Raw shape returned by the GitHub REST API for a pull request.
#[derive(Debug, Deserialize)]
struct ApiPr {
    number: u64,
    title: String,
    html_url: String,
    updated_at: String,
    draft: bool,
    user: ApiUser,
}

#[derive(Debug, Deserialize)]
struct ApiUser {
    login: String,
}

/// Fetch open PRs for a single repository via the GitHub REST API.
pub fn fetch(repo: &str, token: &str) -> Result<FetchResult> {
    debug!("api fetch: {repo}");

    let (owner, name) = repo
        .split_once('/')
        .with_context(|| format!("Invalid repo format: {repo}"))?;

    let client = build_client(token)?;

    let url = format!("{GITHUB_API_BASE}/repos/{owner}/{name}/pulls?state=open&per_page=100");

    let response = client
        .get(&url)
        .send()
        .with_context(|| format!("GitHub API request failed for {repo}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("GitHub API returned {status} for {repo}: {body}");
    }

    let raw: Vec<ApiPr> = response
        .json()
        .with_context(|| format!("Failed to parse GitHub API response for {repo}"))?;

    let prs = raw.into_iter().filter_map(|p| parse_pr(p, repo)).collect();

    Ok(FetchResult {
        repo: repo.to_string(),
        prs,
        source: FetchSource::GitHubApi,
        warning: None,
    })
}

fn parse_pr(p: ApiPr, repo: &str) -> Option<PullRequest> {
    let updated_at = DateTime::parse_from_rfc3339(&p.updated_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .ok()?;

    Some(PullRequest {
        number: p.number,
        title: p.title,
        author: p.user.login,
        repo: repo.to_string(),
        url: p.html_url,
        updated_at,
        draft: p.draft,
    })
}
