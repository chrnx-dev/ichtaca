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

/// Argv for `pass insert`. `-m` reads the multiline body from STDIN.
pub(crate) fn insert_args(path: &str, overwrite: bool) -> Vec<String> {
    let mut args = vec!["insert".to_string(), "-m".to_string()];
    if overwrite {
        args.push("-f".to_string());
    }
    args.push(path.to_string());
    args
}

/// Argv for `pass generate`. `--force` overwrites; `--no-symbols` drops symbols.
pub(crate) fn generate_args(path: &str, len: usize, symbols: bool, overwrite: bool) -> Vec<String> {
    let mut args = vec!["generate".to_string()];
    if !symbols {
        args.push("--no-symbols".to_string());
    }
    if overwrite {
        args.push("--force".to_string());
    }
    args.push(path.to_string());
    args.push(len.to_string());
    args
}

/// Map a failed `pass` invocation's stderr to a specific error.
pub(crate) fn classify_pass_failure(path: &str, stderr: &str) -> PassError {
    if stderr.contains("not in the password store") {
        PassError::EntryNotFound(path.to_string())
    } else if stderr.contains("already") && stderr.contains("exist") {
        PassError::AlreadyExists(path.to_string())
    } else {
        PassError::DecryptFailed {
            entry: path.to_string(),
            message: stderr.trim().to_string(),
        }
    }
}

/// Strip ANSI CSI escape sequences from a line (e.g. colored `pass generate`).
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip until a letter terminates the CSI sequence.
            for c2 in chars.by_ref() {
                if c2.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
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

    /// Run a `pass` subcommand, capturing stdout. Maps failures via stderr.
    fn run_pass(&self, ctx_path: &str, args: &[String]) -> Result<Vec<u8>> {
        use std::process::Command;
        let output = Command::new("pass")
            .args(args)
            .env("PASSWORD_STORE_DIR", &self.store_dir)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(classify_pass_failure(ctx_path, &stderr));
        }
        Ok(output.stdout)
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

    fn insert(&mut self, path: &str, contents: &Secret, overwrite: bool) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        if !overwrite && self.list()?.iter().any(|p| p == path) {
            return Err(PassError::AlreadyExists(path.to_string()));
        }

        let mut child = Command::new("pass")
            .args(insert_args(path, overwrite))
            .env("PASSWORD_STORE_DIR", &self.store_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Secret goes over STDIN, never argv (so it never shows in `ps`).
        child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(contents.expose_bytes())?;

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(classify_pass_failure(path, &stderr));
        }
        Ok(())
    }

    fn edit(&mut self, path: &str) -> Result<()> {
        use std::process::Command;
        // Inherit the terminal so `$EDITOR` (via `pass edit`) runs interactively.
        // The TUI is responsible for suspending/restoring around this call.
        let status = Command::new("pass")
            .arg("edit")
            .arg(path)
            .env("PASSWORD_STORE_DIR", &self.store_dir)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(PassError::DecryptFailed {
                entry: path.to_string(),
                message: "pass edit failed".to_string(),
            })
        }
    }

    fn remove(&mut self, path: &str) -> Result<()> {
        // Best-effort pre-check: align with FakeStore semantics so both return
        // EntryNotFound for a missing entry. This is not TOCTOU-free (the entry
        // could be removed between the list and the rm), but it gives a clear
        // error in the common case rather than silently succeeding with `pass rm -f`.
        if !self.list()?.iter().any(|p| p == path) {
            return Err(PassError::EntryNotFound(path.to_string()));
        }
        self.run_pass(
            path,
            &["rm".to_string(), "-f".to_string(), path.to_string()],
        )
        .map(|_| ())
    }

    fn mv(&mut self, from: &str, to: &str) -> Result<()> {
        self.run_pass(
            from,
            &[
                "mv".to_string(),
                "-f".to_string(),
                from.to_string(),
                to.to_string(),
            ],
        )
        .map(|_| ())
    }

    fn cp(&mut self, from: &str, to: &str) -> Result<()> {
        self.run_pass(
            from,
            &[
                "cp".to_string(),
                "-f".to_string(),
                from.to_string(),
                to.to_string(),
            ],
        )
        .map(|_| ())
    }

    fn generate(&mut self, path: &str, len: usize, symbols: bool) -> Result<Secret> {
        let out = self.run_pass(path, &generate_args(path, len, symbols, true))?;
        // `pass generate` prints the password (often with ANSI). Take the last
        // non-empty line and strip ANSI escapes defensively.
        let text = String::from_utf8_lossy(&out);
        let pw = text
            .lines()
            .rev()
            .map(strip_ansi)
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .unwrap_or_default();
        Ok(Secret::from(format!("{pw}\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::PasswordStore;
    use tempfile::tempdir;

    #[test]
    fn generate_args_respect_overwrite_and_symbols() {
        assert_eq!(
            generate_args("web/x", 24, true, false),
            vec!["generate", "web/x", "24"]
        );
        assert_eq!(
            generate_args("web/x", 24, false, true),
            vec!["generate", "--no-symbols", "--force", "web/x", "24"]
        );
    }

    #[test]
    fn insert_args_force_only_when_overwrite() {
        assert_eq!(insert_args("a/b", false), vec!["insert", "-m", "a/b"]);
        assert_eq!(insert_args("a/b", true), vec!["insert", "-m", "-f", "a/b"]);
    }

    #[test]
    fn classify_pass_failure_maps_missing_entry() {
        let err = classify_pass_failure("web/x", "Error: web/x is not in the password store.");
        assert!(matches!(err, crate::error::PassError::EntryNotFound(_)));
        let err = classify_pass_failure("web/x", "gpg: decryption failed");
        assert!(matches!(err, crate::error::PassError::DecryptFailed { .. }));
    }

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
