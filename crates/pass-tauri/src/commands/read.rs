use serde::Serialize;
use tauri::State;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct EntryMeta {
    pub path: String,
    /// `key: value` fields excluding the password line and the otp uri.
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub has_otp: bool,
}

#[derive(Debug, Serialize)]
pub struct OtpCode {
    pub code: String,
    pub seconds: u64,
}

// ── impl helpers (testable without a Tauri runtime) ──────────────────────────

pub fn list_impl(state: &AppState) -> CommandResult<Vec<String>> {
    let store = state.store.lock().unwrap();
    store.list().map_err(CommandError::from)
}

pub fn show_meta_impl(state: &AppState, path: String) -> CommandResult<EntryMeta> {
    let store = state.store.lock().unwrap();
    let entry = store.show(&path).map_err(CommandError::from)?;

    // Enumerate key:value pairs from the serialized text, skipping:
    //   line 0 (password), any `otpauth://` line, any `@tag`-only line.
    let serialized = entry.serialize();
    let mut fields = Vec::new();
    for line in serialized.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("otpauth://") {
            continue;
        }
        // Skip bare `@tag` lines (all whitespace-separated tokens start with @)
        if !trimmed.is_empty() && trimmed.split_whitespace().all(|tok| tok.starts_with('@')) {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().to_string();
            if !key.is_empty() {
                fields.push((key, val));
            }
        }
    }

    let tags = entry.tags();
    let has_otp = entry.otp_uri().is_some();

    Ok(EntryMeta {
        path,
        fields,
        tags,
        has_otp,
    })
}

pub fn reveal_password_impl(state: &AppState, path: String) -> CommandResult<String> {
    let store = state.store.lock().unwrap();
    let entry = store.show(&path).map_err(CommandError::from)?;
    Ok(entry.password().to_string())
}

pub fn otp_code_impl(state: &AppState, path: String) -> CommandResult<OtpCode> {
    let store = state.store.lock().unwrap();
    let entry = store.show(&path).map_err(CommandError::from)?;
    let uri = entry.otp_uri().ok_or_else(|| CommandError {
        message: "no OTP URI configured for this entry".to_string(),
    })?;
    let otp = passcore::otp::current(uri).map_err(CommandError::from)?;
    Ok(OtpCode {
        code: otp.code,
        seconds: otp.seconds_remaining,
    })
}

pub fn search_fuzzy_impl(state: &AppState, query: String) -> CommandResult<Vec<String>> {
    let store = state.store.lock().unwrap();
    let paths = store.list().map_err(CommandError::from)?;
    let hits = passcore::fuzzy_paths(&query, &paths);
    Ok(hits.into_iter().map(|h| h.path).collect())
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command]
pub fn list(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    list_impl(&state)
}

#[tauri::command]
pub fn show_meta(state: State<'_, AppState>, path: String) -> CommandResult<EntryMeta> {
    show_meta_impl(&state, path)
}

#[tauri::command]
pub fn reveal_password(state: State<'_, AppState>, path: String) -> CommandResult<String> {
    reveal_password_impl(&state, path)
}

#[tauri::command]
pub fn otp_code(state: State<'_, AppState>, path: String) -> CommandResult<OtpCode> {
    otp_code_impl(&state, path)
}

#[tauri::command]
pub fn search_fuzzy(state: State<'_, AppState>, query: String) -> CommandResult<Vec<String>> {
    search_fuzzy_impl(&state, query)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::{Config, FakeStore};

    fn state_with_github() -> AppState {
        let mut store = FakeStore::new();
        store.seed(
            "web/github.com",
            "pw\nuser: bob\notpauth://totp/x?secret=JBSWY3DPEHPK3PXP\n@work\n",
        );
        AppState::new(Box::new(store), Config::default())
    }

    #[test]
    fn list_returns_seeded_paths() {
        let state = state_with_github();
        let paths = list_impl(&state).unwrap();
        assert!(paths.contains(&"web/github.com".to_string()));
    }

    #[test]
    fn show_meta_has_user_field_and_otp_flag() {
        let state = state_with_github();
        let meta = show_meta_impl(&state, "web/github.com".to_string()).unwrap();
        assert!(meta.has_otp);
        assert!(meta.fields.iter().any(|(k, v)| k == "user" && v == "bob"));
    }

    #[test]
    fn show_meta_excludes_password() {
        let state = state_with_github();
        let meta = show_meta_impl(&state, "web/github.com".to_string()).unwrap();
        // No field value should be the password string "pw"
        for (k, v) in &meta.fields {
            assert_ne!(v, "pw", "field ({k}: {v}) exposes the password");
        }
        // The password must not appear as a key either
        for (k, _) in &meta.fields {
            assert_ne!(k, "pw");
        }
    }

    #[test]
    fn show_meta_excludes_otp_uri_from_fields() {
        let state = state_with_github();
        let meta = show_meta_impl(&state, "web/github.com".to_string()).unwrap();
        for (k, v) in &meta.fields {
            assert!(
                !v.starts_with("otpauth://"),
                "field ({k}: {v}) exposes OTP URI"
            );
        }
    }

    #[test]
    fn show_meta_tags_contains_work() {
        let state = state_with_github();
        let meta = show_meta_impl(&state, "web/github.com".to_string()).unwrap();
        assert!(
            meta.tags.contains(&"work".to_string()),
            "tags: {:?}",
            meta.tags
        );
    }

    #[test]
    fn reveal_password_returns_correct_password() {
        let state = state_with_github();
        let pw = reveal_password_impl(&state, "web/github.com".to_string()).unwrap();
        assert_eq!(pw, "pw");
    }

    #[test]
    fn otp_code_returns_six_chars_and_valid_seconds() {
        let state = state_with_github();
        let otp = otp_code_impl(&state, "web/github.com".to_string()).unwrap();
        assert_eq!(otp.code.len(), 6, "OTP code should be 6 digits");
        assert!(
            otp.seconds <= 30,
            "seconds_remaining should be <= period (30)"
        );
        assert!(otp.seconds > 0, "seconds_remaining should be > 0");
    }

    #[test]
    fn otp_code_missing_otp_returns_err() {
        let mut store = FakeStore::new();
        store.seed("email/work", "pw\nuser: alice\n");
        let state = AppState::new(Box::new(store), Config::default());
        let result = otp_code_impl(&state, "email/work".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("no OTP"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn search_fuzzy_finds_github() {
        let state = state_with_github();
        let hits = search_fuzzy_impl(&state, "git".to_string()).unwrap();
        assert!(
            hits.contains(&"web/github.com".to_string()),
            "search 'git' should find web/github.com; got: {:?}",
            hits
        );
    }

    #[test]
    fn search_fuzzy_empty_returns_all() {
        let state = state_with_github();
        let hits = search_fuzzy_impl(&state, String::new()).unwrap();
        assert!(!hits.is_empty());
    }

    #[test]
    fn otp_code_does_not_leak_uri_or_secret() {
        let state = state_with_github();
        let otp = otp_code_impl(&state, "web/github.com".to_string()).unwrap();
        let serialized = serde_json::to_string(&otp).expect("OtpCode must serialize");
        assert!(
            !serialized.contains("otpauth"),
            "serialized OtpCode leaks OTP URI: {serialized}"
        );
        assert!(
            !serialized.contains("JBSWY3DPEHPK3PXP"),
            "serialized OtpCode leaks OTP secret: {serialized}"
        );
    }
}
