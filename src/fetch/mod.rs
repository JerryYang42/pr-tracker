pub mod api;
pub mod gh;

use crate::config::Config;
use crate::model::{FetchResult, PullRequest};
use tracing::{info, warn};

/// Fetch PRs for every configured repository.
///
/// Strategy per repo:
/// 1. Try `gh` CLI.
/// 2. If that fails and `use_api_fallback` is true and a token is available,
///    fall back to the GitHub REST API.
/// 3. Otherwise record the error as a warning and continue.
///
/// Always returns partial results — never aborts the whole fetch for one repo failure.
pub fn fetch_all(config: &Config) -> Vec<FetchResult> {
    let token = if config.github.use_api_fallback {
        std::env::var(&config.github.token_env).ok()
    } else {
        None
    };

    let mut results = Vec::new();

    for repo in &config.github.repos {
        match gh::fetch(repo) {
            Ok(result) => {
                info!("gh: fetched {} PR(s) from {repo}", result.prs.len());
                results.push(result);
            }
            Err(gh_err) => {
                warn!("gh failed for {repo}: {gh_err:#}");

                if let Some(tok) = &token {
                    match api::fetch(repo, tok) {
                        Ok(mut result) => {
                            result.warning =
                                Some(format!("gh failed, using API fallback: {gh_err:#}"));
                            info!("api: fetched {} PR(s) from {repo}", result.prs.len());
                            results.push(result);
                        }
                        Err(api_err) => {
                            warn!("API also failed for {repo}: {api_err:#}");
                            results.push(FetchResult {
                                repo: repo.clone(),
                                prs: vec![],
                                source: crate::model::FetchSource::GitHubApi,
                                warning: Some(format!(
                                    "Both gh and API failed — gh: {gh_err:#} | api: {api_err:#}"
                                )),
                            });
                        }
                    }
                } else {
                    results.push(FetchResult {
                        repo: repo.clone(),
                        prs: vec![],
                        source: crate::model::FetchSource::GhCli,
                        warning: Some(format!("gh failed (no API fallback): {gh_err:#}")),
                    });
                }
            }
        }
    }

    results
}

/// Flatten and sort all PR results by most recently updated, descending.
pub fn sorted_prs(results: &[FetchResult]) -> Vec<&PullRequest> {
    let mut prs: Vec<&PullRequest> = results.iter().flat_map(|r| r.prs.iter()).collect();
    prs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    prs
}
