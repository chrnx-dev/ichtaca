//! Git status and sync commands for the desktop app.
//!
//! Unlike the TUI, the webview has no terminal to prompt on, so sync runs with
//! interactive prompts disabled (`passcore::git::sync_quiet`). Auth that needs a
//! passphrase fails fast with git's own message; the frontend tells the user to
//! run `ichtaca git push` from a shell instead of spinning forever.

use std::path::PathBuf;

use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

/// Resolve the store dir for git. `None` when there is nothing real to act on:
/// no store was built at startup, or this is the labeled demo fake.
fn git_dir(state: &AppState) -> Option<PathBuf> {
    if state.demo || state.init_error.is_some() {
        return None;
    }
    Some(passcore::store_dir(state.config.store_dir.clone()))
}

// ── impl helpers (testable without a Tauri runtime) ──────────────────────────

/// Local git state, or `None` when the store is not a git repo — the frontend
/// hides the whole git chip in that case.
pub fn git_status_impl(state: &AppState) -> Option<passcore::git::Status> {
    passcore::git::status(&git_dir(state)?)
}

pub fn git_sync_impl(state: &AppState, op: passcore::git::Op) -> CommandResult<String> {
    let dir = git_dir(state).ok_or_else(|| CommandError {
        message: "no git repository for this store".to_string(),
    })?;
    passcore::git::sync_quiet(&dir, op).map_err(CommandError::from)
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_status(state: State<'_, AppState>) -> Option<passcore::git::Status> {
    git_status_impl(&state)
}

#[tauri::command]
pub fn git_sync(state: State<'_, AppState>, op: passcore::git::Op) -> CommandResult<String> {
    git_sync_impl(&state, op)
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::{Config, FakeStore};

    #[test]
    fn demo_and_uninitialized_states_have_no_git() {
        let demo = AppState::new_demo(Box::new(FakeStore::demo()), Config::default());
        assert!(
            git_dir(&demo).is_none(),
            "demo store must not borrow a repo"
        );
        assert!(git_status_impl(&demo).is_none());

        let dead = AppState::uninitialized("no pass".into(), Config::default());
        assert!(git_dir(&dead).is_none());
        assert!(git_sync_impl(&dead, passcore::git::Op::Push).is_err());
    }

    #[test]
    fn non_repo_store_reports_no_status() {
        let cfg = Config {
            store_dir: Some(PathBuf::from("/nonexistent/ichtaca-test-store")),
            ..Config::default()
        };
        let state = AppState::new(Box::new(FakeStore::new()), cfg);
        assert!(git_status_impl(&state).is_none());
    }
}
