//! The abstract store interface and its domain types.

pub mod cli;
pub mod fake;

use crate::entry::Entry;
use crate::error::{PassError, Result};
use crate::secret::Secret;

/// A node in the entry tree. A directory has `path == None` and children;
/// a leaf entry has `path == Some(full/path)` and no children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryNode {
    pub name: String,
    pub path: Option<String>,
    pub children: Vec<EntryNode>,
}

impl EntryNode {
    pub fn is_leaf(&self) -> bool {
        self.path.is_some() && self.children.is_empty()
    }

    /// Build a sorted tree from flat entry paths like `web/github.com`.
    pub fn from_paths(paths: &[String]) -> Vec<EntryNode> {
        let mut roots: Vec<EntryNode> = Vec::new();
        for full in paths {
            let segments: Vec<&str> = full.split('/').collect();
            insert_path(&mut roots, &segments, full);
        }
        sort_nodes(&mut roots);
        roots
    }
}

fn insert_path(level: &mut Vec<EntryNode>, segments: &[&str], full: &str) {
    let (head, rest) = match segments.split_first() {
        Some(v) => v,
        None => return,
    };
    let is_last = rest.is_empty();
    let pos = level.iter().position(|n| n.name == *head);
    let idx = match pos {
        Some(i) => i,
        None => {
            level.push(EntryNode {
                name: head.to_string(),
                path: if is_last {
                    Some(full.to_string())
                } else {
                    None
                },
                children: Vec::new(),
            });
            level.len() - 1
        }
    };
    if !is_last {
        insert_path(&mut level[idx].children, rest, full);
    }
}

fn sort_nodes(nodes: &mut [EntryNode]) {
    nodes.sort_by(|a, b| a.name.cmp(&b.name));
    for n in nodes.iter_mut() {
        sort_nodes(&mut n.children);
    }
}

/// Abstract access to a password store. Implemented by `PassCliStore` (real)
/// and `FakeStore` (tests). Future: `NativeStore` / `HybridStore`.
pub trait PasswordStore {
    /// All entry paths, sorted (e.g. `web/github.com`). No decryption.
    fn list(&self) -> Result<Vec<String>>;

    /// Decrypt a single entry, returning the raw secret (no parsing).
    fn show_raw(&self, path: &str) -> Result<Secret>;

    /// Decrypt and parse a single entry. Errors with `PassError::Parse` if the
    /// decrypted bytes are not valid UTF-8 (rather than silently emptying them).
    /// Provided default — do not remove.
    fn show(&self, path: &str) -> Result<Entry> {
        let secret = self.show_raw(path)?;
        let text = std::str::from_utf8(secret.expose_bytes())
            .map_err(|e| PassError::Parse(format!("entry `{path}` is not valid UTF-8: {e}")))?;
        Ok(Entry::parse(text))
    }

    /// Create or overwrite an entry with the given raw decrypted contents.
    /// The secret is passed to `pass` over STDIN, never argv. When `overwrite`
    /// is false and the entry already exists, returns `PassError::AlreadyExists`.
    fn insert(&mut self, path: &str, contents: &Secret, overwrite: bool) -> Result<()>;

    /// Edit an entry interactively, delegating to `pass edit` / `$EDITOR`.
    /// In a TUI this requires suspending the terminal; the core only shells out.
    fn edit(&mut self, path: &str) -> Result<()>;

    /// Remove an entry.
    fn remove(&mut self, path: &str) -> Result<()>;

    /// Move/rename an entry (`pass mv`).
    fn mv(&mut self, from: &str, to: &str) -> Result<()>;

    /// Copy an entry (`pass cp`).
    fn cp(&mut self, from: &str, to: &str) -> Result<()>;

    /// Generate a new password of `len` chars at `path` and return it.
    /// `symbols == false` corresponds to `pass generate --no-symbols`.
    fn generate(&mut self, path: &str, len: usize, symbols: bool) -> Result<Secret>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::secret::Secret;
    use crate::store::fake::FakeStore;

    #[test]
    fn trait_object_exposes_write_methods() {
        // Compile-time proof that writes are part of the trait surface and are
        // callable through a trait object.
        let mut store = FakeStore::new();
        store.insert("web/x", &Secret::from("pw\n"), false).unwrap();
        let dyn_store: &dyn PasswordStore = &store;
        assert_eq!(dyn_store.list().unwrap(), vec!["web/x".to_string()]);
    }

    #[test]
    fn builds_tree_from_flat_paths() {
        let paths = vec![
            "email/work".to_string(),
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
        ];
        let tree = EntryNode::from_paths(&paths);
        // top level: email/, web/ (sorted, dirs are just names)
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].name, "email");
        assert_eq!(tree[1].name, "web");
        assert_eq!(tree[1].children.len(), 2);
        assert_eq!(tree[1].children[0].name, "github.com");
        assert_eq!(tree[1].children[0].path.as_deref(), Some("web/github.com"));
        assert!(tree[1].children[0].is_leaf());
    }

    #[test]
    fn tree_helper_on_store_output() {
        // A store's flat list converts to a tree via EntryNode::from_paths.
        let paths = vec!["a/b".to_string(), "a/c".to_string()];
        let tree = EntryNode::from_paths(&paths);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "a");
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn default_show_rejects_non_utf8() {
        use crate::error::{PassError, Result};
        use crate::secret::Secret;

        struct BinaryStore;
        impl PasswordStore for BinaryStore {
            fn list(&self) -> Result<Vec<String>> {
                Ok(vec![])
            }
            fn show_raw(&self, _path: &str) -> Result<Secret> {
                Ok(Secret::new(vec![0xff, 0xfe, 0xfd]))
            }
            fn insert(&mut self, _path: &str, _contents: &Secret, _overwrite: bool) -> Result<()> {
                unimplemented!()
            }
            fn edit(&mut self, _path: &str) -> Result<()> {
                unimplemented!()
            }
            fn remove(&mut self, _path: &str) -> Result<()> {
                unimplemented!()
            }
            fn mv(&mut self, _from: &str, _to: &str) -> Result<()> {
                unimplemented!()
            }
            fn cp(&mut self, _from: &str, _to: &str) -> Result<()> {
                unimplemented!()
            }
            fn generate(&mut self, _path: &str, _len: usize, _symbols: bool) -> Result<Secret> {
                unimplemented!()
            }
        }

        let err = BinaryStore.show("x").unwrap_err();
        assert!(matches!(err, PassError::Parse(_)));
    }
}
