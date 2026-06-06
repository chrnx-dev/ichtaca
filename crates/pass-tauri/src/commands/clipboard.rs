use std::sync::Arc;
use std::time::Duration;

use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ── impl helper (testable without a Tauri runtime) ───────────────────────────

pub fn copy_password_impl(state: &AppState, path: String) -> CommandResult<()> {
    // Fetch the raw secret first — this must succeed even when no clipboard
    // is available, and is what we test on headless CI.
    let secret = {
        let store = state.store();
        store.show_raw(&path).map_err(CommandError::from)?
    };

    let backend = passcore::clipboard::default_backend().map_err(|e| CommandError {
        message: e.to_string(),
    })?;

    let timeout = Duration::from_secs(state.config.clipboard.clear_after);
    passcore::clipboard::copy_and_autoclear(Arc::from(backend), &secret, timeout).map_err(|e| {
        CommandError {
            message: e.to_string(),
        }
    })
}

// ── Tauri command wrapper ─────────────────────────────────────────────────────

#[tauri::command]
pub fn copy_password(state: State<'_, AppState>, path: String) -> CommandResult<()> {
    copy_password_impl(&state, path)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::{Config, FakeStore};

    fn make_state_with(path: &str, contents: &str) -> AppState {
        let mut store = FakeStore::new();
        store.seed(path, contents);
        AppState::new(Box::new(store), Config::default())
    }

    /// A missing entry must always return Err, regardless of clipboard
    /// availability.
    #[test]
    fn copy_password_missing_entry_returns_err() {
        let state = AppState::new(Box::new(FakeStore::new()), Config::default());
        let result = copy_password_impl(&state, "no/such".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        // The error should mention the missing path, not panic.
        assert!(
            err.message.contains("no/such") || err.message.to_lowercase().contains("not found"),
            "unexpected error message: {}",
            err.message
        );
    }

    /// Happy path: if a clipboard backend is available, the copy succeeds.
    /// On headless CI the backend may be absent — we tolerate that gracefully
    /// (the error is from the backend, not a panic or logic bug).
    #[test]
    fn copy_password_happy_path_or_tolerates_missing_backend() {
        let state = make_state_with("web/github.com", "mypw\nuser: bob\n");
        let result = copy_password_impl(&state, "web/github.com".to_string());
        match result {
            Ok(()) => {
                // Real clipboard available — all good.
            }
            Err(e) => {
                // Headless CI: the store fetch succeeded but the clipboard
                // backend was absent or failed. This is acceptable.
                // What is NOT acceptable: an error that says "entry not found"
                // (that would mean the secret fetch is broken).
                assert!(
                    !e.message.contains("web/github.com"),
                    "unexpected store error on happy path: {}",
                    e.message
                );
                // The error should be backend-related.
                eprintln!(
                    "copy_password_happy_path: clipboard backend unavailable ({}); tolerated on headless CI",
                    e.message
                );
            }
        }
    }
}
