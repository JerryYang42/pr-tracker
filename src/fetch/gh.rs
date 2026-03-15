use crate::model::{FetchResult, FetchSource, PullRequest};
use anyhow::Result;
use chrono::DateTime;
use serde::Deserialize;
use std::process::Command;
use tracing::{debug, warn};

/// Fields we request from `gh pr list --json`.
const GH_JSON_FIELDS: &str = "number,title,author,url,updatedAt,isDraft";

/// Raw shape returned by `gh pr list --json`.
#[derive(Debug, Deserialize)]
struct GhPr {
    number: u64,
    title: String,
    author: GhAuthor,
    url: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
}

#[derive(Debug, Deserialize)]
struct GhAuthor {
    login: String,
}

/// Fetch open PRs for a single repository using the `gh` CLI.
/// Returns `Err` when the command fails so the caller can decide on fallback.
pub fn fetch(repo: &str) -> Result<FetchResult> {
    debug!("gh fetch: {repo}");

    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--json",
            GH_JSON_FIELDS,
            "--limit",
            "100",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr list failed for {repo}: {stderr}");
    }

    let raw: Vec<GhPr> = serde_json::from_slice(&output.stdout)?;
    let prs = raw.into_iter().filter_map(|p| parse_pr(p, repo)).collect();

    Ok(FetchResult {
        repo: repo.to_string(),
        prs,
        source: FetchSource::GhCli,
        warning: None,
    })
}

fn parse_pr(p: GhPr, repo: &str) -> Option<PullRequest> {
    let updated_at = DateTime::parse_from_rfc3339(&p.updated_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| {
            warn!(
                "Skipping PR #{} — bad date '{}': {e}",
                p.number, p.updated_at
            )
        })
        .ok()?;

    Some(PullRequest {
        number: p.number,
        title: p.title,
        author: p.author.login,
        repo: repo.to_string(),
        url: p.url,
        updated_at,
        draft: p.is_draft,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_gh_json() {
        let json = r#"[
          {
            "number": 42,
            "title": "Fix flaky test",
            "author": { "login": "alice" },
            "url": "https://github.com/owner/repo/pull/42",
            "updatedAt": "2026-03-14T09:00:00Z",
            "isDraft": false
          }
        ]"#;

        let raw: Vec<GhPr> = serde_json::from_str(json).unwrap();
        assert_eq!(raw.len(), 1);
        let pr = parse_pr(raw.into_iter().next().unwrap(), "owner/repo").unwrap();
        assert_eq!(pr.number, 42);
        assert_eq!(pr.author, "alice");
        assert!(!pr.draft);
    }

    #[test]
    fn skips_pr_with_invalid_date() {
        let raw = GhPr {
            number: 1,
            title: "Bad date".into(),
            author: GhAuthor {
                login: "bob".into(),
            },
            url: "https://github.com/owner/repo/pull/1".into(),
            updated_at: "not-a-date".into(),
            is_draft: false,
        };
        assert!(parse_pr(raw, "owner/repo").is_none());
    }
}
