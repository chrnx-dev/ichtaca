//! Two-level search: fast fuzzy over entry paths (no GPG) and an explicit deep
//! search that decrypts entries via the store and matches content + tags.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config as MatcherConfig, Matcher, Utf32Str};

use crate::error::Result;
use crate::store::PasswordStore;

/// A fuzzy path hit with its match score (higher is better).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathHit {
    pub path: String,
    pub score: u32,
}

/// Fuzzy-match `query` against `paths`, returning hits sorted best-first.
/// An empty query returns every path (score 0), preserving input order.
pub fn fuzzy_paths(query: &str, paths: &[String]) -> Vec<PathHit> {
    if query.is_empty() {
        return paths
            .iter()
            .map(|p| PathHit {
                path: p.clone(),
                score: 0,
            })
            .collect();
    }

    let mut matcher = Matcher::new(MatcherConfig::DEFAULT.match_paths());
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut buf = Vec::new();
    let mut hits: Vec<PathHit> = paths
        .iter()
        .filter_map(|p| {
            let haystack = Utf32Str::new(p, &mut buf);
            pattern.score(haystack, &mut matcher).map(|score| PathHit {
                path: p.clone(),
                score,
            })
        })
        .collect();

    // Sort by score descending, then path ascending for stable ties.
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
    hits
}

/// Decrypt every entry via `store` and return the paths whose content, fields,
/// tags, or path contain `query` (case-insensitive substring). Slower: invokes
/// the store's `show` per entry (GPG for a real store).
pub fn deep(query: &str, store: &dyn PasswordStore) -> Result<Vec<String>> {
    let needle = query.to_lowercase();
    let mut matches = Vec::new();
    for path in store.list()? {
        if path.to_lowercase().contains(&needle) {
            matches.push(path);
            continue;
        }
        let entry = store.show(&path)?;
        let in_body = entry.serialize().to_lowercase().contains(&needle);
        let in_tags = entry
            .tags()
            .iter()
            .any(|t| t.to_lowercase().contains(&needle));
        if in_body || in_tags {
            matches.push(path);
        }
    }
    matches.sort();
    matches.dedup();
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::fake::FakeStore;

    fn paths() -> Vec<String> {
        vec![
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
            "email/work".to_string(),
            "servers/prod-db".to_string(),
        ]
    }

    #[test]
    fn fuzzy_matches_subsequence_and_ranks_best_first() {
        let hits = fuzzy_paths("gthb", &paths());
        assert_eq!(
            hits.first().map(|h| h.path.as_str()),
            Some("web/github.com")
        );
    }

    #[test]
    fn fuzzy_empty_query_returns_all_unranked() {
        let hits = fuzzy_paths("", &paths());
        assert_eq!(hits.len(), 4);
    }

    #[test]
    fn fuzzy_no_match_returns_empty() {
        let hits = fuzzy_paths("zzzzzz", &paths());
        assert!(hits.is_empty());
    }

    #[test]
    fn deep_matches_content_and_tags() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "pw\nuser: octocat\n@work\n");
        store.seed("email/work", "pw\nuser: bob\n");
        store.seed("servers/prod", "pw\nhost: db.internal\n");

        // matches a field value
        let hits = deep("octocat", &store).unwrap();
        assert_eq!(hits, vec!["web/github.com".to_string()]);

        // matches a tag
        let hits = deep("work", &store).unwrap();
        assert!(hits.contains(&"web/github.com".to_string()));
        assert!(hits.contains(&"email/work".to_string())); // path match too

        // matches content host
        let hits = deep("internal", &store).unwrap();
        assert_eq!(hits, vec!["servers/prod".to_string()]);
    }
}
