//! `PassCliStore` — wraps the `pass` CLI for `show` and walks the store dir for
//! `list`. Listing needs no GPG; only `show` decrypts.

use std::path::{Path, PathBuf};

use crate::error::{PassError, Result};

/// Resolve the store directory: explicit override > `$PASSWORD_STORE_DIR` >
/// `~/.password-store`. `env` is injected for testing.
pub(crate) fn resolve_store_dir(
    override_dir: Option<PathBuf>,
    env: impl Fn(&str) -> Option<String>,
) -> PathBuf {
    if let Some(d) = override_dir {
        return d;
    }
    if let Some(v) = env("PASSWORD_STORE_DIR") {
        return PathBuf::from(v);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".password-store")
}

/// Error if the store directory does not exist.
pub(crate) fn ensure_store_exists(dir: &Path) -> Result<()> {
    if dir.is_dir() {
        Ok(())
    } else {
        Err(PassError::StoreNotFound(dir.to_path_buf()))
    }
}

/// Error if `pass` or `gpg` are not on PATH.
pub(crate) fn ensure_binaries_present() -> Result<()> {
    which::which("pass").map_err(|_| PassError::PassNotInstalled)?;
    which::which("gpg").map_err(|_| PassError::GpgNotInstalled)?;
    Ok(())
}

use walkdir::WalkDir;

use crate::secret::Secret;
use crate::store::PasswordStore;

/// A store backed by the `pass` CLI and the on-disk store directory.
#[derive(Debug, Clone)]
pub struct PassCliStore {
    store_dir: PathBuf,
}

impl PassCliStore {
    /// Construct using config override / env / default, validating presence of
    /// the store directory and the `pass`/`gpg` binaries.
    pub fn detect(override_dir: Option<PathBuf>) -> Result<Self> {
        ensure_binaries_present()?;
        let store_dir = resolve_store_dir(override_dir, |k| std::env::var(k).ok());
        ensure_store_exists(&store_dir)?;
        Ok(Self { store_dir })
    }

    /// Construct against an explicit directory without validation (used in tests
    /// and when the caller has already validated).
    pub fn with_store_dir(store_dir: PathBuf) -> Self {
        Self { store_dir }
    }
}

impl PasswordStore for PassCliStore {
    fn list(&self) -> Result<Vec<String>> {
        let mut paths = Vec::new();
        for entry in WalkDir::new(&self.store_dir)
            .into_iter()
            .filter_entry(|e| e.file_name() != ".git")
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("gpg") {
                continue;
            }
            let rel = match p.strip_prefix(&self.store_dir) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let trimmed = rel_str.strip_suffix(".gpg").unwrap_or(&rel_str).to_string();
            paths.push(trimmed);
        }
        paths.sort();
        Ok(paths)
    }

    fn show_raw(&self, path: &str) -> Result<Secret> {
        use std::process::Command;

        let output = Command::new("pass")
            .arg("show")
            .arg(path)
            .env("PASSWORD_STORE_DIR", &self.store_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // `pass` prints "is not in the password store" for missing entries.
            if stderr.contains("not in the password store") {
                return Err(PassError::EntryNotFound(path.to_string()));
            }
            return Err(PassError::DecryptFailed {
                entry: path.to_string(),
                message: stderr.trim().to_string(),
            });
        }
        Ok(Secret::new(output.stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::PasswordStore;
    use tempfile::tempdir;

    #[test]
    fn store_dir_defaults_to_password_store_under_home() {
        // When no override and no env var, path ends with `.password-store`.
        let dir = resolve_store_dir(None, |_| None);
        assert!(dir.ends_with(".password-store"));
    }

    #[test]
    fn store_dir_prefers_explicit_override() {
        let dir = resolve_store_dir(Some("/tmp/store".into()), |_| None);
        assert_eq!(dir, std::path::PathBuf::from("/tmp/store"));
    }

    #[test]
    fn store_dir_uses_env_when_no_override() {
        let dir = resolve_store_dir(None, |k| {
            (k == "PASSWORD_STORE_DIR").then(|| "/env/store".to_string())
        });
        assert_eq!(dir, std::path::PathBuf::from("/env/store"));
    }

    #[test]
    fn missing_store_dir_is_reported() {
        let missing = tempdir().unwrap().path().join("does-not-exist");
        let err = ensure_store_exists(&missing).unwrap_err();
        assert!(matches!(err, crate::error::PassError::StoreNotFound(_)));
    }

    #[test]
    fn lists_gpg_files_as_sorted_paths() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("web")).unwrap();
        std::fs::write(root.join("web/github.com.gpg"), b"x").unwrap();
        std::fs::write(root.join("web/gitlab.com.gpg"), b"x").unwrap();
        std::fs::write(root.join("email.gpg"), b"x").unwrap();
        // a non-.gpg file and the .git dir must be ignored
        std::fs::write(root.join("README.md"), b"x").unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join(".git/config.gpg"), b"x").unwrap();

        let store = PassCliStore::with_store_dir(root.to_path_buf());
        assert_eq!(
            store.list().unwrap(),
            vec!["email", "web/github.com", "web/gitlab.com"]
        );
    }

    #[test]
    fn list_strips_only_one_gpg_suffix() {
        // An entry literally named "weird.gpg" is stored as "weird.gpg.gpg".
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("weird.gpg.gpg"), b"x").unwrap();
        let store = PassCliStore::with_store_dir(root.to_path_buf());
        assert_eq!(store.list().unwrap(), vec!["weird.gpg"]);
    }
}
