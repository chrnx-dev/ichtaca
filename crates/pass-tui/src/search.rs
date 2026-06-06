//! Fuzzy search bar state. Filtering delegates to `passcore::search::fuzzy_paths`
//! so ranking lives in the core; this struct only holds query + results.

/// The query string and the current filtered/ranked result paths.
#[derive(Debug, Default, Clone)]
pub struct SearchState {
    pub query: String,
    /// Ranked result paths (best first). Empty query = all paths, store order.
    pub results: Vec<String>,
    /// Index into `results` of the highlighted item.
    pub cursor: usize,
}

impl SearchState {
    pub fn push(&mut self, c: char) {
        self.query.push(c);
        self.cursor = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.cursor = 0;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.cursor = 0;
    }

    /// Recompute `results` from the full path list given the current query.
    pub fn recompute(&mut self, all_paths: &[String]) {
        if self.query.is_empty() {
            self.results = all_paths.to_vec();
        } else {
            // `fuzzy_paths` returns `Vec<PathHit>` sorted best-first.
            // `PathHit` has a `.path: String` field.
            self.results = passcore::search::fuzzy_paths(&self.query, all_paths)
                .into_iter()
                .map(|m| m.path)
                .collect();
        }
        if self.cursor >= self.results.len() {
            self.cursor = self.results.len().saturating_sub(1);
        }
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.results.get(self.cursor).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> Vec<String> {
        vec![
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
            "email/work".to_string(),
        ]
    }

    #[test]
    fn empty_query_yields_all_paths() {
        let mut s = SearchState::default();
        s.recompute(&paths());
        assert_eq!(s.results.len(), 3);
    }

    #[test]
    fn typing_filters_results() {
        let mut s = SearchState::default();
        s.push('g');
        s.push('i');
        s.push('t');
        s.recompute(&paths());
        // both github.com and gitlab.com contain "git"
        assert!(s.results.iter().all(|p| p.contains("git")));
        assert!(s.results.iter().any(|p| p == "web/github.com"));
    }

    #[test]
    fn backspace_widens_results() {
        let mut s = SearchState::default();
        s.push('z'); // matches nothing
        s.recompute(&paths());
        assert!(s.results.is_empty());
        s.backspace();
        s.recompute(&paths());
        assert_eq!(s.results.len(), 3);
    }

    #[test]
    fn query_text_tracks_input() {
        let mut s = SearchState::default();
        s.push('a');
        s.push('b');
        s.backspace();
        assert_eq!(s.query, "a");
    }
}
