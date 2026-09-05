//! Git status and sync for a `pass` store, by shelling out to `git`.
//!
//! `pass` already auto-commits every `insert`/`rm`/`mv`/`cp`/`generate` when the
//! store is a git repo, so there is nothing here to stage or commit — only
//! "am I ahead of my remote?" plus an explicit pull/push.
//!
//! ponytail: shelling out to `git`, not `git2`/libgit2. A whole C library for
//! one `status` parse and two network commands is not a trade worth making;
//! add the crate if we ever need in-process conflict resolution.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{PassError, Result};

/// Local git state of the store.
///
/// Everything here is read from local refs — **nothing touches the network**.
/// That makes `ahead` and `dirty` always accurate, and `behind` accurate only
/// as of the last fetch/pull. Callers should not present `behind` as live.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Status {
    /// Current branch name, or `(detached)`.
    pub branch: String,
    /// Commits on the local branch not on its upstream.
    pub ahead: u32,
    /// Commits on the upstream not local, as of the last fetch.
    pub behind: u32,
    /// Number of files with uncommitted changes (including untracked).
    pub dirty: u32,
    /// Whether the branch tracks an upstream. Without one there is nowhere to
    /// push, so frontends should offer setup guidance rather than a sync key.
    pub upstream: bool,
}

/// A network sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Pull,
    Push,
}

impl Op {
    fn args(self) -> &'static [&'static str] {
        match self {
            // --rebase keeps the store's history linear; a conflict aborts and
            // is left for the user to resolve with git directly.
            Op::Pull => &["pull", "--rebase"],
            Op::Push => &["push"],
        }
    }

    /// Human label used in error messages and UI.
    pub fn label(self) -> &'static str {
        match self {
            Op::Pull => "pull",
            Op::Push => "push",
        }
    }
}

/// Is `store_dir` a git repository? (`.git` may be a dir or a worktree file.)
pub fn is_repo(store_dir: &Path) -> bool {
    store_dir.join(".git").exists()
}

/// Read the store's local git state.
///
/// Returns `None` when the store is not a git repo or `git` is unavailable —
/// frontends hide their git UI entirely in that case rather than showing an
/// error the user did not ask for.
pub fn status(store_dir: &Path) -> Option<Status> {
    if !is_repo(store_dir) {
        return None;
    }
    let out = capture(store_dir, &["status", "--porcelain=v2", "--branch"]).ok()?;
    Some(parse_status(&out))
}

/// Run `op` with git's stdio inherited, so SSH passphrase and credential
/// prompts reach the user's terminal.
///
/// A TUI caller **must** suspend the alternate screen first (see the
/// raw-edit suspension in `pass-tui`). Not usable from a GUI — use
/// [`sync_quiet`] there.
pub fn sync(store_dir: &Path, op: Op) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(store_dir)
        .args(op.args())
        .status()
        .map_err(|e| PassError::GitError(format!("could not run git: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(PassError::GitError(format!("git {} failed", op.label())))
    }
}

/// Run `op` capturing output, with interactive prompts disabled.
///
/// For callers with no terminal (the desktop app). Auth that needs a prompt
/// fails fast with git's own message instead of hanging forever on a tty
/// nobody can see; the frontend tells the user to run it from a shell.
pub fn sync_quiet(store_dir: &Path, op: Op) -> Result<String> {
    capture(store_dir, op.args())
}

/// Run a git subcommand in the store, capturing stdout and never prompting.
fn capture(store_dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(store_dir)
        .args(args)
        // Never block on a credential prompt we cannot show or answer.
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| PassError::GitError(format!("could not run git: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let msg = if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        };
        return Err(PassError::GitError(msg));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Parse `git status --porcelain=v2 --branch` output.
///
/// Header lines start with `#`; every other non-empty line is one changed or
/// untracked file.
fn parse_status(out: &str) -> Status {
    let mut s = Status::default();
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            s.branch = rest.trim().to_string();
        } else if line.starts_with("# branch.upstream ") {
            s.upstream = true;
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            let mut counts = rest.split_whitespace();
            s.ahead = counts.next().and_then(parse_count).unwrap_or(0);
            s.behind = counts.next().and_then(parse_count).unwrap_or(0);
        } else if !line.starts_with('#') && !line.trim().is_empty() {
            s.dirty += 1;
        }
    }
    s
}

/// `+2` / `-1` → `2` / `1`.
fn parse_count(token: &str) -> Option<u32> {
    token.trim_start_matches(['+', '-']).parse().ok()
}

/// Setup guidance for a store that is not yet under git. Frontends show this
/// instead of a sync UI; there is no in-app wizard because this is a one-time,
/// two-command action.
pub fn setup_hint(store_dir: &Path) -> String {
    format!(
        "Store at {} is not under git — no sync.\nEnable it with:\n\n  \
         pass git init\n  pass git remote add origin <url>\n  pass git push -u origin main\n",
        store_dir.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ahead_behind_and_dirty() {
        let out = "\
# branch.oid deadbeef
# branch.head main
# branch.upstream origin/main
# branch.ab +2 -1
1 .M N... 100644 100644 100644 aaa bbb web/github.com.gpg
? new/entry.gpg
";
        let s = parse_status(out);
        assert_eq!(s.branch, "main");
        assert!(s.upstream);
        assert_eq!(s.ahead, 2);
        assert_eq!(s.behind, 1);
        assert_eq!(s.dirty, 2, "one modified + one untracked");
    }

    #[test]
    fn clean_repo_without_upstream() {
        let out = "# branch.oid deadbeef\n# branch.head main\n";
        let s = parse_status(out);
        assert_eq!(s.branch, "main");
        assert!(
            !s.upstream,
            "no branch.upstream line means no remote branch"
        );
        assert_eq!((s.ahead, s.behind, s.dirty), (0, 0, 0));
    }

    #[test]
    fn non_repo_has_no_status() {
        let dir = std::env::temp_dir().join("ichtaca-not-a-repo");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!is_repo(&dir));
        assert!(status(&dir).is_none());
    }
}
