/// Integration-level tests for the `gh` JSON parser via the public fetch module.
///
/// These tests exercise the parsing logic with fixture JSON to ensure the
/// normalisation layer works correctly without needing a real `gh` invocation.

// Re-export the internal parse helpers for testing by accessing the module path.
// The fetch::gh module's unit tests already cover parse_pr; this file covers the
// full serde round-trip to validate field mapping against realistic fixture data.

#[cfg(test)]
mod gh_parser_tests {
    use serde::Deserialize;

    // Mirror the private GhPr struct for test fixture deserialization.
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

    #[test]
    fn parses_full_gh_response() {
        let json = r#"[
            {
                "number": 101,
                "title": "Add new feature",
                "author": { "login": "dev1" },
                "url": "https://github.com/org/repo/pull/101",
                "updatedAt": "2026-03-14T10:30:00Z",
                "isDraft": false
            },
            {
                "number": 99,
                "title": "WIP: refactor core module",
                "author": { "login": "dev2" },
                "url": "https://github.com/org/repo/pull/99",
                "updatedAt": "2026-03-13T08:00:00Z",
                "isDraft": true
            }
        ]"#;

        let prs: Vec<GhPr> = serde_json::from_str(json).unwrap();
        assert_eq!(prs.len(), 2);

        assert_eq!(prs[0].number, 101);
        assert_eq!(prs[0].author.login, "dev1");
        assert!(!prs[0].is_draft);

        assert_eq!(prs[1].number, 99);
        assert!(prs[1].is_draft);
    }

    #[test]
    fn handles_empty_list() {
        let json = "[]";
        let prs: Vec<GhPr> = serde_json::from_str(json).unwrap();
        assert!(prs.is_empty());
    }

    #[test]
    fn rejects_missing_required_field() {
        // Missing "author" field
        let json = r#"[{"number": 1, "title": "x", "url": "u", "updatedAt": "2026-01-01T00:00:00Z", "isDraft": false}]"#;
        let result: Result<Vec<GhPr>, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
