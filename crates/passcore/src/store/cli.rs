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

#[cfg(test)]
mod tests {
    use super::*;
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
}
