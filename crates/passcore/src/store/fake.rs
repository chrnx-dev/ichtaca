//! An in-memory `PasswordStore` for testing consumers without `pass`/`gpg`.

use std::collections::BTreeMap;

use crate::error::{PassError, Result};
use crate::secret::Secret;
use crate::store::PasswordStore;

#[derive(Debug, Default)]
pub struct FakeStore {
    entries: BTreeMap<String, String>,
}

impl FakeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or overwrite an entry with the given raw decrypted text.
    pub fn insert(&mut self, path: &str, contents: &str) {
        self.entries.insert(path.to_string(), contents.to_string());
    }
}

impl PasswordStore for FakeStore {
    fn list(&self) -> Result<Vec<String>> {
        Ok(self.entries.keys().cloned().collect())
    }

    fn show_raw(&self, path: &str) -> Result<Secret> {
        self.entries
            .get(path)
            .map(|c| Secret::from(c.as_str()))
            .ok_or_else(|| PassError::EntryNotFound(path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::PasswordStore;

    #[test]
    fn lists_inserted_paths_sorted() {
        let mut s = FakeStore::new();
        s.insert("web/github.com", "pw1\nuser: bob\n");
        s.insert("email/work", "pw2\n");
        assert_eq!(s.list().unwrap(), vec!["email/work", "web/github.com"]);
    }

    #[test]
    fn shows_and_parses_entry() {
        let mut s = FakeStore::new();
        s.insert("web/github.com", "pw1\nuser: bob\n");
        let e = s.show("web/github.com").unwrap();
        assert_eq!(e.password(), "pw1");
        assert_eq!(e.field("user"), Some("bob"));
    }

    #[test]
    fn missing_entry_is_error() {
        let s = FakeStore::new();
        assert!(s.show("nope").is_err());
    }
}
