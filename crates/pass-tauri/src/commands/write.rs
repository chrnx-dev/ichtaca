use serde::Deserialize;
use tauri::State;

use passcore::Secret;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

/// Form payload from the UI for *inserting* a new entry.
/// `password` is line 1; `fields` are `key: value` rows;
/// `otp` is an optional `otpauth://` line; `tags` join as `@a @b`.
#[derive(Debug, Deserialize)]
pub struct EntryInput {
    pub password: String,
    pub fields: Vec<(String, String)>,
    pub otp: Option<String>,
    pub tags: Vec<String>,
}

/// Form payload from the UI for *updating* an existing entry.
/// Only `password` and `fields` are accepted — OTP and tags are preserved
/// unchanged from the existing entry text.
// NOTE: editing OTP/tags on an existing entry is a Phase-3 prerequisite —
// it needs passcore `Entry::set_otp`/`set_tags`; until then update preserves
// the existing otpauth/@tags lines.
#[derive(Debug, Deserialize)]
pub struct UpdateInput {
    pub password: String,
    pub fields: Vec<(String, String)>,
}

/// Build entry text from scratch (used by `insert`).
fn build_entry_text(input: &EntryInput) -> String {
    let mut lines = Vec::new();
    lines.push(input.password.clone());
    for (k, v) in &input.fields {
        lines.push(format!("{k}: {v}"));
    }
    if let Some(otp) = &input.otp {
        lines.push(otp.clone());
    }
    if !input.tags.is_empty() {
        let tag_line = input
            .tags
            .iter()
            .map(|t| format!("@{t}"))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(tag_line);
    }
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

// ── impl helpers (testable without a Tauri runtime) ──────────────────────────

pub fn insert_impl(
    state: &AppState,
    path: String,
    input: EntryInput,
    overwrite: bool,
) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    let text = build_entry_text(&input);
    let secret = Secret::from(text.as_str());
    store
        .insert(&path, &secret, overwrite)
        .map_err(CommandError::from)
}

pub fn update_entry_impl(state: &AppState, path: String, input: UpdateInput) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    // Load existing entry, apply structured changes (preserves unknown lines,
    // including existing otpauth:// and @tags lines).
    let mut entry = store.show(&path).map_err(CommandError::from)?;
    entry.set_password(&input.password);
    for (k, v) in &input.fields {
        entry.set_field(k, v);
    }
    let text = entry.serialize();
    let secret = Secret::from(text.as_str());
    store
        .insert(&path, &secret, true)
        .map_err(CommandError::from)
}

pub fn remove_impl(state: &AppState, path: String) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    store.remove(&path).map_err(CommandError::from)
}

pub fn mv_impl(state: &AppState, from: String, to: String) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    store.mv(&from, &to).map_err(CommandError::from)
}

pub fn cp_impl(state: &AppState, from: String, to: String) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    store.cp(&from, &to).map_err(CommandError::from)
}

pub fn generate_impl(
    state: &AppState,
    path: String,
    len: usize,
    symbols: bool,
) -> CommandResult<()> {
    let mut store = state.store.lock().unwrap();
    store
        .generate(&path, len, symbols)
        .map(|_| ())
        .map_err(CommandError::from)
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command]
pub fn insert(
    state: State<'_, AppState>,
    path: String,
    input: EntryInput,
    overwrite: bool,
) -> CommandResult<()> {
    insert_impl(&state, path, input, overwrite)
}

#[tauri::command]
pub fn update_entry(
    state: State<'_, AppState>,
    path: String,
    input: UpdateInput,
) -> CommandResult<()> {
    update_entry_impl(&state, path, input)
}

#[tauri::command]
pub fn remove(state: State<'_, AppState>, path: String) -> CommandResult<()> {
    remove_impl(&state, path)
}

#[tauri::command]
pub fn mv(state: State<'_, AppState>, from: String, to: String) -> CommandResult<()> {
    mv_impl(&state, from, to)
}

#[tauri::command]
pub fn cp(state: State<'_, AppState>, from: String, to: String) -> CommandResult<()> {
    cp_impl(&state, from, to)
}

#[tauri::command]
pub fn generate(
    state: State<'_, AppState>,
    path: String,
    len: usize,
    symbols: bool,
) -> CommandResult<()> {
    generate_impl(&state, path, len, symbols)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::{Config, FakeStore};

    fn make_state() -> AppState {
        AppState::new(Box::new(FakeStore::new()), Config::default())
    }

    fn make_entry_input(password: &str, fields: Vec<(&str, &str)>, tags: Vec<&str>) -> EntryInput {
        EntryInput {
            password: password.to_string(),
            fields: fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            otp: None,
            tags: tags.into_iter().map(|t| t.to_string()).collect(),
        }
    }

    fn make_update_input(password: &str, fields: Vec<(&str, &str)>) -> UpdateInput {
        UpdateInput {
            password: password.to_string(),
            fields: fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    // ── insert ────────────────────────────────────────────────────────────────

    #[test]
    fn insert_creates_entry() {
        let state = make_state();
        let input = make_entry_input("secret", vec![("user", "bob")], vec![]);
        insert_impl(&state, "web/x".to_string(), input, false).unwrap();
        let store = state.store.lock().unwrap();
        let entry = store.show("web/x").unwrap();
        assert_eq!(entry.password(), "secret");
        assert_eq!(entry.field("user"), Some("bob"));
    }

    #[test]
    fn insert_overwrite_false_twice_returns_already_exists() {
        let state = make_state();
        let input1 = make_entry_input("pw1", vec![], vec![]);
        insert_impl(&state, "web/x".to_string(), input1, false).unwrap();
        let input2 = make_entry_input("pw2", vec![], vec![]);
        let err = insert_impl(&state, "web/x".to_string(), input2, false).unwrap_err();
        assert!(
            err.message.to_lowercase().contains("exist"),
            "expected 'exist' in error: {}",
            err.message
        );
    }

    #[test]
    fn insert_with_otp_and_tags() {
        let state = make_state();
        let input = EntryInput {
            password: "mypw".to_string(),
            fields: vec![("user".to_string(), "alice".to_string())],
            otp: Some("otpauth://totp/x?secret=JBSWY3DPEHPK3PXP".to_string()),
            tags: vec!["work".to_string(), "personal".to_string()],
        };
        insert_impl(&state, "new/entry".to_string(), input, false).unwrap();
        let store = state.store.lock().unwrap();
        let entry = store.show("new/entry").unwrap();
        assert!(entry.otp_uri().is_some());
        assert!(entry.tags().contains(&"work".to_string()));
        assert!(entry.tags().contains(&"personal".to_string()));
    }

    // ── update_entry ──────────────────────────────────────────────────────────

    #[test]
    fn update_entry_preserves_unknown_lines() {
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\nrandom note\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("newpw", vec![("user", "alice")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store.lock().unwrap();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.password(), "newpw");
        assert_eq!(entry.field("user"), Some("alice"));
        // Unknown line must survive the round-trip.
        assert!(
            entry.serialize().contains("random note"),
            "unknown line was lost; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_preserves_otp_and_note_lines() {
        let mut store = FakeStore::new();
        store.seed(
            "web/site",
            "pw\nuser: bob\notpauth://totp/x?secret=JBSWY3DPEHPK3PXP\nrandom note\n",
        );
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("newpw", vec![("user", "alice")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store.lock().unwrap();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.password(), "newpw");
        assert_eq!(entry.field("user"), Some("alice"));
        // OTP line must survive the round-trip.
        assert!(
            entry.otp_uri().is_some(),
            "OTP URI was lost after update; serialized: {:?}",
            entry.serialize()
        );
        // Unknown note line must also survive.
        assert!(
            entry.serialize().contains("random note"),
            "note line was lost after update; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_updates_password_and_field() {
        let mut store = FakeStore::new();
        store.seed("web/site", "oldpw\nuser: bob\nurl: a.com\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("newpw", vec![("user", "carol")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store.lock().unwrap();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.password(), "newpw");
        assert_eq!(entry.field("user"), Some("carol"));
        // url field untouched
        assert_eq!(entry.field("url"), Some("a.com"));
    }

    // ── remove ────────────────────────────────────────────────────────────────

    #[test]
    fn remove_deletes_the_entry() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\n");
        let state = AppState::new(Box::new(store), Config::default());

        remove_impl(&state, "web/x".to_string()).unwrap();

        let store = state.store.lock().unwrap();
        assert!(store.show("web/x").is_err());
    }

    #[test]
    fn remove_missing_entry_returns_err() {
        let state = make_state();
        assert!(remove_impl(&state, "no/such".to_string()).is_err());
    }

    // ── mv ────────────────────────────────────────────────────────────────────

    #[test]
    fn mv_moves_entry() {
        let mut store = FakeStore::new();
        store.seed("web/old", "pw\n");
        let state = AppState::new(Box::new(store), Config::default());

        mv_impl(&state, "web/old".to_string(), "web/new".to_string()).unwrap();

        let store = state.store.lock().unwrap();
        assert!(store.show("web/old").is_err());
        assert_eq!(store.show("web/new").unwrap().password(), "pw");
    }

    // ── cp ────────────────────────────────────────────────────────────────────

    #[test]
    fn cp_duplicates_entry() {
        let mut store = FakeStore::new();
        store.seed("web/src", "pw\n");
        let state = AppState::new(Box::new(store), Config::default());

        cp_impl(&state, "web/src".to_string(), "web/dst".to_string()).unwrap();

        let store = state.store.lock().unwrap();
        assert_eq!(store.show("web/src").unwrap().password(), "pw");
        assert_eq!(store.show("web/dst").unwrap().password(), "pw");
    }

    // ── generate ─────────────────────────────────────────────────────────────

    #[test]
    fn generate_writes_entry_of_requested_length() {
        let state = make_state();
        generate_impl(&state, "new/gen".to_string(), 20, false).unwrap();
        let store = state.store.lock().unwrap();
        let entry = store.show("new/gen").unwrap();
        assert_eq!(
            entry.password().chars().count(),
            20,
            "generated password should be 20 chars"
        );
    }

    #[test]
    fn generate_with_symbols_succeeds() {
        let state = make_state();
        generate_impl(&state, "new/sym".to_string(), 16, true).unwrap();
        let store = state.store.lock().unwrap();
        assert_eq!(
            store.show("new/sym").unwrap().password().chars().count(),
            16
        );
    }
}
