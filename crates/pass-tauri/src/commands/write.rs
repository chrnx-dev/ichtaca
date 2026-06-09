use serde::Deserialize;
use tauri::State;

use passcore::Secret;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

/// Form payload from the UI for *inserting* a new entry.
/// `password` is line 1; `fields` are `key: value` rows;
/// `otp` is an optional `otpauth://` line; `tags` join as `@a @b`.
#[derive(Deserialize)]
pub struct EntryInput {
    pub password: String,
    pub fields: Vec<(String, String)>,
    pub otp: Option<String>,
    pub tags: Vec<String>,
}

impl std::fmt::Debug for EntryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryInput")
            .field("password", &"<redacted>")
            .field("fields", &self.fields)
            .field("otp", &self.otp)
            .field("tags", &self.tags)
            .finish()
    }
}

/// Form payload from the UI for *updating* an existing entry.
/// `password` is line 1; `fields` are `key: value` rows;
/// `otp` is an optional `otpauth://` line (None clears it); `tags` replace existing tags.
#[derive(Deserialize)]
pub struct UpdateInput {
    pub password: String,
    pub fields: Vec<(String, String)>,
    pub otp: Option<String>,
    pub tags: Vec<String>,
}

impl std::fmt::Debug for UpdateInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateInput")
            .field("password", &"<redacted>")
            .field("fields", &self.fields)
            .field("otp", &"<redacted>")
            .field("tags", &self.tags)
            .finish()
    }
}

/// Build entry text from scratch (used by `insert`).
fn build_entry_text(input: &EntryInput) -> String {
    let mut lines = Vec::new();
    lines.push(input.password.clone());
    for (k, v) in &input.fields {
        let k = k.trim();
        if !k.is_empty() {
            lines.push(format!("{k}: {v}"));
        }
    }
    if let Some(otp) = &input.otp {
        lines.push(otp.clone());
    }
    if !input.tags.is_empty() {
        let tag_line = input
            .tags
            .iter()
            .map(|t| format!("@{}", t.trim_start_matches('@')))
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
    let mut store = state.store();
    let text = build_entry_text(&input);
    let secret = Secret::from(text.as_str());
    store
        .insert(&path, &secret, overwrite)
        .map_err(CommandError::from)
}

pub fn update_entry_impl(state: &AppState, path: String, input: UpdateInput) -> CommandResult<()> {
    let mut store = state.store();
    // Load existing entry, apply structured changes (preserves unknown lines).
    let mut entry = store.show(&path).map_err(CommandError::from)?;

    // (1) Collect existing field keys from the original entry.
    let existing_keys = entry.field_keys();

    // (2) Build the set of keys present in the input (normalized: trimmed).
    let input_keys: std::collections::HashSet<&str> =
        input.fields.iter().map(|(k, _)| k.trim()).collect();

    // (3) Remove fields that exist in the entry but are absent from the input.
    for key in &existing_keys {
        if !input_keys.contains(key.trim()) {
            entry.remove_field(key.trim());
        }
    }

    // (4-6) Apply the standard mutations. Field keys are trimmed so a key like
    // " user" matches the existing "user" field instead of deleting it and
    // creating a spurious " user" line.
    entry.set_password(&input.password);
    for (k, v) in &input.fields {
        let k = k.trim();
        if !k.is_empty() {
            entry.set_field(k, v);
        }
    }
    entry.set_otp(input.otp.as_deref());
    entry.set_tags(&input.tags);

    // (7) Serialize and persist.
    let text = entry.serialize();
    let secret = Secret::from(text.as_str());
    store
        .insert(&path, &secret, true)
        .map_err(CommandError::from)
}

pub fn remove_impl(state: &AppState, path: String) -> CommandResult<()> {
    let mut store = state.store();
    store.remove(&path).map_err(CommandError::from)
}

pub fn mv_impl(state: &AppState, from: String, to: String) -> CommandResult<()> {
    let mut store = state.store();
    store.mv(&from, &to).map_err(CommandError::from)
}

pub fn cp_impl(state: &AppState, from: String, to: String) -> CommandResult<()> {
    let mut store = state.store();
    store.cp(&from, &to).map_err(CommandError::from)
}

pub fn generate_impl(
    state: &AppState,
    path: String,
    len: usize,
    symbols: bool,
) -> CommandResult<()> {
    let mut store = state.store();
    store
        .generate(&path, len, symbols)
        .map(|_| ())
        .map_err(CommandError::from)
}

/// Generate a fresh random password using the configured length/charset and
/// return it to the form. This is a *new* value the user is about to enter and
/// save — not an existing stored secret — so returning it over IPC is fine.
pub fn generate_password_impl(state: &AppState) -> CommandResult<String> {
    let cfg = &state.config.generator;
    Ok(passcore::generate_password(cfg.length, cfg.symbols))
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

#[tauri::command]
pub fn generate_password(state: State<'_, AppState>) -> CommandResult<String> {
    generate_password_impl(&state)
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
            otp: None,
            tags: vec![],
        }
    }

    // ── insert ────────────────────────────────────────────────────────────────

    #[test]
    fn insert_creates_entry() {
        let state = make_state();
        let input = make_entry_input("secret", vec![("user", "bob")], vec![]);
        insert_impl(&state, "web/x".to_string(), input, false).unwrap();
        let store = state.store();
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
        let store = state.store();
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

        let store = state.store();
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
    fn update_entry_preserves_note_line_when_editing_field() {
        // Updating only password/fields (otp=None clears otp; tags=[] clears tags),
        // but an unrelated free-text note line must survive the round-trip.
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\nrandom note\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("newpw", vec![("user", "alice")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.password(), "newpw");
        assert_eq!(entry.field("user"), Some("alice"));
        // Unknown note line must survive.
        assert!(
            entry.serialize().contains("random note"),
            "note line was lost after update; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_sets_new_otp() {
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = UpdateInput {
            password: "pw".to_string(),
            fields: vec![],
            otp: Some("otpauth://totp/x?secret=NEW".to_string()),
            tags: vec![],
        };
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(
            entry.otp_uri(),
            Some("otpauth://totp/x?secret=NEW"),
            "OTP URI not set; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_clears_otp() {
        let mut store = FakeStore::new();
        store.seed(
            "web/site",
            "pw\nuser: bob\notpauth://totp/x?secret=OLD\nrandom note\n",
        );
        let state = AppState::new(Box::new(store), Config::default());

        let input = UpdateInput {
            password: "pw".to_string(),
            fields: vec![],
            otp: None,
            tags: vec![],
        };
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert!(
            entry.otp_uri().is_none(),
            "OTP URI should be cleared; serialized: {:?}",
            entry.serialize()
        );
        // Unrelated free-text line must still be present.
        assert!(
            entry.serialize().contains("random note"),
            "note line was lost after clearing OTP; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_sets_tags() {
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = UpdateInput {
            password: "pw".to_string(),
            fields: vec![],
            otp: None,
            tags: vec!["work".to_string(), "home".to_string()],
        };
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        let tags = entry.tags();
        assert!(
            tags.contains(&"work".to_string()),
            "missing 'work' tag; tags: {:?}",
            tags
        );
        assert!(
            tags.contains(&"home".to_string()),
            "missing 'home' tag; tags: {:?}",
            tags
        );
    }

    #[test]
    fn update_entry_updates_password_and_field() {
        // When url IS included in the input, it is updated; when omitted it is deleted.
        // This test keeps url in the input to verify update-in-place still works.
        let mut store = FakeStore::new();
        store.seed("web/site", "oldpw\nuser: bob\nurl: a.com\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("newpw", vec![("user", "carol"), ("url", "a.com")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.password(), "newpw");
        assert_eq!(entry.field("user"), Some("carol"));
        // url was included in input so it must still be present
        assert_eq!(entry.field("url"), Some("a.com"));
    }

    // ── field deletion ────────────────────────────────────────────────────────

    #[test]
    fn update_entry_deletes_omitted_field() {
        // Seed: pw, user, url, plus a bare note line.
        // Update with only user in fields (url omitted) → url must be gone,
        // user updated, and the free-text note preserved.
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\nurl: x\nnote\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = make_update_input("pw", vec![("user", "alice")]);
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.field("user"), Some("alice"), "user should be updated");
        assert_eq!(entry.field("url"), None, "url should have been deleted");
        assert!(
            entry.serialize().contains("note"),
            "free-text note must survive; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_free_text_line_preserved_independent_of_fields() {
        // A bare note line (no `key: value` structure) is never treated as a
        // field key; it must survive even when all named fields are removed.
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\nnote\n");
        let state = AppState::new(Box::new(store), Config::default());

        // No fields in input → user is removed; note (non-field) must survive.
        let input = UpdateInput {
            password: "pw".to_string(),
            fields: vec![],
            otp: None,
            tags: vec![],
        };
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(entry.field("user"), None, "user should have been deleted");
        assert!(
            entry.serialize().contains("note"),
            "free-text note must be preserved; serialized: {:?}",
            entry.serialize()
        );
    }

    #[test]
    fn update_entry_trims_field_key_whitespace_updates_in_place() {
        // An input field key with surrounding whitespace (" user") must be
        // normalized so it matches the existing `user` field: update in place,
        // do NOT delete it, and do NOT create a spurious " user" line.
        let mut store = FakeStore::new();
        store.seed("web/site", "pw\nuser: bob\n");
        let state = AppState::new(Box::new(store), Config::default());

        let input = UpdateInput {
            password: "pw".to_string(),
            fields: vec![(" user".into(), "alice".into())],
            otp: None,
            tags: vec![],
        };
        update_entry_impl(&state, "web/site".to_string(), input).unwrap();

        let store = state.store();
        let entry = store.show("web/site").unwrap();
        assert_eq!(
            entry.field("user"),
            Some("alice"),
            "user must be updated in place; serialized: {:?}",
            entry.serialize()
        );
        let serialized = entry.serialize();
        let user_lines = serialized
            .lines()
            .filter(|l| l.split_once(':').is_some_and(|(k, _)| k.trim() == "user"))
            .count();
        assert_eq!(
            user_lines, 1,
            "must not create a duplicate ` user` line; serialized: {serialized:?}"
        );
    }

    // ── remove ────────────────────────────────────────────────────────────────

    #[test]
    fn remove_deletes_the_entry() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\n");
        let state = AppState::new(Box::new(store), Config::default());

        remove_impl(&state, "web/x".to_string()).unwrap();

        let store = state.store();
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

        let store = state.store();
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

        let store = state.store();
        assert_eq!(store.show("web/src").unwrap().password(), "pw");
        assert_eq!(store.show("web/dst").unwrap().password(), "pw");
    }

    // ── generate ─────────────────────────────────────────────────────────────

    #[test]
    fn insert_tags_strip_leading_at_prefix() {
        // Tags sent as "@work" and "home" must both be stored without double-@.
        let state = make_state();
        let input = EntryInput {
            password: "pw".to_string(),
            fields: vec![],
            otp: None,
            tags: vec!["@work".to_string(), "home".to_string()],
        };
        insert_impl(&state, "web/tagged".to_string(), input, false).unwrap();
        let store = state.store();
        let entry = store.show("web/tagged").unwrap();
        let text = entry.serialize();
        assert!(
            text.contains("@work"),
            "expected @work in serialized text: {:?}",
            text
        );
        assert!(
            text.contains("@home"),
            "expected @home in serialized text: {:?}",
            text
        );
        assert!(
            !text.contains("@@work"),
            "double-@ prefix found in serialized text: {:?}",
            text
        );
    }

    #[test]
    fn generate_writes_entry_of_requested_length() {
        let state = make_state();
        generate_impl(&state, "new/gen".to_string(), 20, false).unwrap();
        let store = state.store();
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
        let store = state.store();
        assert_eq!(
            store.show("new/sym").unwrap().password().chars().count(),
            16
        );
    }

    // ── generate_password (returns a value, driven by config) ─────────────────

    #[test]
    fn generate_password_uses_config_length_and_charset() {
        // Custom config: length 32, no symbols → must yield a 32-char
        // alphanumeric-only password.
        let mut config = Config::default();
        config.generator.length = 32;
        config.generator.symbols = false;
        let state = AppState::new(Box::new(FakeStore::new()), config);

        let pw = generate_password_impl(&state).unwrap();
        assert_eq!(pw.chars().count(), 32, "must honour configured length");
        assert!(
            pw.chars().all(|c| c.is_ascii_alphanumeric()),
            "symbols=false must yield alphanumeric-only password; got {pw:?}"
        );
    }

    #[test]
    fn generate_password_default_config_length_20() {
        let state = make_state();
        let pw = generate_password_impl(&state).unwrap();
        assert_eq!(pw.chars().count(), 20, "default length is 20");
    }
}
