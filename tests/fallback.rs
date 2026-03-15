/// Tests for fetch orchestration and fallback behaviour.

#[cfg(test)]
mod orchestration_tests {
    use pr_tracker::fetch::sorted_prs;
    use pr_tracker::model::{FetchResult, FetchSource, PullRequest};

    fn make_pr(number: u64, repo: &str, ts: &str) -> PullRequest {
        PullRequest {
            number,
            title: format!("PR #{number}"),
            author: "alice".into(),
            repo: repo.into(),
            url: format!("https://github.com/{repo}/pull/{number}"),
            updated_at: ts.parse().unwrap(),
            draft: false,
        }
    }

    #[test]
    fn sorted_prs_orders_by_updated_desc() {
        let results = vec![
            FetchResult {
                repo: "org/a".into(),
                prs: vec![
                    make_pr(1, "org/a", "2026-03-10T00:00:00Z"),
                    make_pr(2, "org/a", "2026-03-14T00:00:00Z"),
                ],
                source: FetchSource::GhCli,
                warning: None,
            },
            FetchResult {
                repo: "org/b".into(),
                prs: vec![make_pr(3, "org/b", "2026-03-12T00:00:00Z")],
                source: FetchSource::GhCli,
                warning: None,
            },
        ];

        let prs = sorted_prs(&results);
        assert_eq!(prs.len(), 3);
        // Most recent first
        assert_eq!(prs[0].number, 2);
        assert_eq!(prs[1].number, 3);
        assert_eq!(prs[2].number, 1);
    }

    #[test]
    fn sorted_prs_empty_results() {
        let results: Vec<FetchResult> = vec![];
        assert!(sorted_prs(&results).is_empty());
    }

    #[test]
    fn fetch_result_with_warning_preserved() {
        let result = FetchResult {
            repo: "org/r".into(),
            prs: vec![],
            source: FetchSource::GitHubApi,
            warning: Some("gh failed, using API fallback: auth error".into()),
        };
        assert!(result.warning.is_some());
    }
}
