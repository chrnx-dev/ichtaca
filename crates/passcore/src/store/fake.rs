//! An in-memory `PasswordStore` for testing consumers without `pass`/`gpg`.

use std::collections::BTreeMap;

use crate::entry::Entry;
use crate::error::{PassError, Result};
use crate::secret::Secret;
use crate::store::PasswordStore;

#[derive(Debug, Default)]
pub struct FakeStore {
    entries: BTreeMap<String, String>,
    counter: std::cell::Cell<u64>,
}

impl FakeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or overwrite an entry with the given raw decrypted text.
    /// Convenience used by tests that don't need the trait's `insert`.
    pub fn seed(&mut self, path: &str, contents: &str) {
        self.entries.insert(path.to_string(), contents.to_string());
    }

    /// A deterministic pseudo-password so tests are stable. Not cryptographic;
    /// `FakeStore` never guards real secrets.
    fn fake_password(&self, len: usize, symbols: bool) -> String {
        const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const SYMS: &[u8] = b"!@#$%^&*()-_=+";
        let pool: Vec<u8> = if symbols {
            ALNUM.iter().chain(SYMS.iter()).copied().collect()
        } else {
            ALNUM.to_vec()
        };
        let seed = self.counter.get();
        self.counter.set(seed + 1);
        (0..len)
            .map(|i| {
                let idx =
                    (seed.wrapping_mul(2654435761).wrapping_add(i as u64)) as usize % pool.len();
                pool[idx] as char
            })
            .collect()
    }
}

impl PasswordStore for FakeStore {
    fn list(&self) -> Result<Vec<String>> {
        Ok(self.entries.keys().cloned().collect())
    }

    fn show(&self, path: &str) -> Result<Entry> {
        Ok(Entry::parse(self.show_raw(path)?.expose_str()))
    }

    fn show_raw(&self, path: &str) -> Result<Secret> {
        self.entries
            .get(path)
            .map(|c| Secret::from(c.as_str()))
            .ok_or_else(|| PassError::EntryNotFound(path.to_string()))
    }

    fn insert(&mut self, path: &str, contents: &Secret, overwrite: bool) -> Result<()> {
        if !overwrite && self.entries.contains_key(path) {
            return Err(PassError::AlreadyExists(path.to_string()));
        }
        self.entries
            .insert(path.to_string(), contents.expose_str().to_string());
        Ok(())
    }

    fn edit(&mut self, path: &str) -> Result<()> {
        // No interactive editor in tests; just assert the entry exists.
        if self.entries.contains_key(path) {
            Ok(())
        } else {
            Err(PassError::EntryNotFound(path.to_string()))
        }
    }

    fn remove(&mut self, path: &str) -> Result<()> {
        self.entries
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| PassError::EntryNotFound(path.to_string()))
    }

    fn mv(&mut self, from: &str, to: &str) -> Result<()> {
        let contents = self
            .entries
            .remove(from)
            .ok_or_else(|| PassError::EntryNotFound(from.to_string()))?;
        self.entries.insert(to.to_string(), contents);
        Ok(())
    }

    fn cp(&mut self, from: &str, to: &str) -> Result<()> {
        let contents = self
            .entries
            .get(from)
            .cloned()
            .ok_or_else(|| PassError::EntryNotFound(from.to_string()))?;
        self.entries.insert(to.to_string(), contents);
        Ok(())
    }

    fn generate(&mut self, path: &str, len: usize, symbols: bool) -> Result<Secret> {
        let pw = self.fake_password(len, symbols);
        self.entries.insert(path.to_string(), format!("{pw}\n"));
        Ok(Secret::from(format!("{pw}\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::PasswordStore;

    #[test]
    fn lists_inserted_paths_sorted() {
        let mut s = FakeStore::new();
        s.seed("web/github.com", "pw1\nuser: bob\n");
        s.seed("email/work", "pw2\n");
        assert_eq!(s.list().unwrap(), vec!["email/work", "web/github.com"]);
    }

    #[test]
    fn shows_and_parses_entry() {
        let mut s = FakeStore::new();
        s.seed("web/github.com", "pw1\nuser: bob\n");
        let e = s.show("web/github.com").unwrap();
        assert_eq!(e.password(), "pw1");
        assert_eq!(e.field("user"), Some("bob"));
    }

    #[test]
    fn missing_entry_is_error() {
        let s = FakeStore::new();
        assert!(s.show("nope").is_err());
    }

    use crate::secret::Secret;

    #[test]
    fn insert_creates_then_refuses_overwrite_unless_allowed() {
        let mut s = FakeStore::new();
        s.insert("web/x", &Secret::from("pw1\n"), false).unwrap();
        assert_eq!(s.show("web/x").unwrap().password(), "pw1");
        // second insert without overwrite fails
        let err = s
            .insert("web/x", &Secret::from("pw2\n"), false)
            .unwrap_err();
        assert!(matches!(err, crate::error::PassError::AlreadyExists(_)));
        // with overwrite it succeeds
        s.insert("web/x", &Secret::from("pw2\n"), true).unwrap();
        assert_eq!(s.show("web/x").unwrap().password(), "pw2");
    }

    #[test]
    fn remove_deletes_the_entry() {
        let mut s = FakeStore::new();
        s.insert("a", &Secret::from("p\n"), false).unwrap();
        s.remove("a").unwrap();
        assert!(s.show("a").is_err());
        assert!(s.remove("a").is_err());
    }

    #[test]
    fn mv_moves_and_cp_duplicates() {
        let mut s = FakeStore::new();
        s.insert("a", &Secret::from("p\n"), false).unwrap();
        s.mv("a", "b").unwrap();
        assert!(s.show("a").is_err());
        assert_eq!(s.show("b").unwrap().password(), "p");
        s.cp("b", "c").unwrap();
        assert_eq!(s.show("b").unwrap().password(), "p");
        assert_eq!(s.show("c").unwrap().password(), "p");
    }

    #[test]
    fn generate_writes_and_returns_a_password_of_requested_len() {
        let mut s = FakeStore::new();
        let secret = s.generate("new/x", 20, true).unwrap();
        assert_eq!(secret.first_line().chars().count(), 20);
        // the generated password is now the entry's first line
        assert_eq!(s.show("new/x").unwrap().password().chars().count(), 20);
    }

    #[test]
    fn edit_on_fake_is_a_noop_ok() {
        let mut s = FakeStore::new();
        s.insert("a", &Secret::from("p\n"), false).unwrap();
        assert!(s.edit("a").is_ok());
    }
}
