//! The create/edit form. Built from a template (new) or a parsed `Entry` (edit).
//! Editing routes through `Entry` so unrecognized lines round-trip unchanged.

use passcore::Entry;

/// Entry templates for seeding new-entry forms with suggested field keys.
/// This is a local TUI enum; it is intentionally separate from `passcore::Template`
/// (a struct used for store-level template config).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Login,
    // Consumed by Task 10 (CRUD transitions) and Task 12 (UI template picker).
    #[allow(dead_code)]
    OAuth,
    #[allow(dead_code)]
    Server,
    #[allow(dead_code)]
    Note,
    #[allow(dead_code)]
    Blank,
}

/// Suggested field keys for a template.
fn template_keys(t: Template) -> &'static [&'static str] {
    match t {
        Template::Login => &["user", "url"],
        Template::OAuth => &["client_id", "client_secret", "url"],
        Template::Server => &["host", "user", "port"],
        Template::Note => &[],
        Template::Blank => &[],
    }
}

/// One editable field row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormField {
    pub key: String,
    pub value: String,
}

/// A create/edit form. `base` holds the parsed entry for edits (to preserve
/// unknown lines); for new entries it starts empty.
#[derive(Debug, Clone)]
pub struct Form {
    // `path` and `editing` are used by `update.rs` (Task 10) to build `SideEffect::Save`.
    // `focus` is used by the form-edit widget (Task 10/12).
    #[allow(dead_code)]
    pub path: String,
    pub password: String,
    pub fields: Vec<FormField>,
    /// Index of the focused field (0 = password, 1.. = fields).
    #[allow(dead_code)]
    pub focus: usize,
    /// Whether we are editing an existing entry (drives overwrite + base reuse).
    #[allow(dead_code)]
    pub editing: bool,
    /// For edits: the original entry, so unknown lines survive.
    base: Option<Entry>,
}

impl Form {
    /// Build a form pre-populated from an existing entry.
    pub fn from_entry(path: &str, entry: &Entry) -> Self {
        let mut fields = Vec::new();
        for key in entry_field_keys(entry) {
            if let Some(v) = entry.field(&key) {
                fields.push(FormField {
                    key,
                    value: v.to_string(),
                });
            }
        }
        Self {
            path: path.to_string(),
            password: entry.password().to_string(),
            fields,
            focus: 0,
            editing: true,
            base: Some(entry.clone()),
        }
    }

    /// Build a blank form seeded with the suggested keys for a template.
    pub fn new_from_template(path: &str, template: Template) -> Self {
        let fields = template_keys(template)
            .iter()
            .map(|k| FormField {
                key: (*k).to_string(),
                value: String::new(),
            })
            .collect();
        Self {
            path: path.to_string(),
            password: String::new(),
            fields,
            focus: 0,
            editing: false,
            base: None,
        }
    }

    /// Return the current value for a named field, if present.
    // Used by tests and by app.rs (Task 13); allow until the runtime is wired.
    #[allow(dead_code)]
    pub fn field_value(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.key == key)
            .map(|f| f.value.as_str())
    }

    /// Update a field value (in place if the key exists, or append a new row).
    // Used by tests and by app.rs (Task 13); allow until the runtime is wired.
    #[allow(dead_code)]
    pub fn set_field(&mut self, key: &str, value: &str) {
        if let Some(f) = self.fields.iter_mut().find(|f| f.key == key) {
            f.value = value.to_string();
        } else {
            self.fields.push(FormField {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
    }

    /// Serialize back to raw entry text.
    ///
    /// For **edits**: clone the original `Entry`, apply password and field
    /// mutations via `set_password`/`set_field` (which touch only the targeted
    /// lines), then `serialize()`. This guarantees unrecognized lines survive.
    ///
    /// For **new entries**: emit `password\n` followed by non-empty `key: value`
    /// lines only.
    pub fn to_contents(&self) -> String {
        match &self.base {
            Some(base) => {
                let mut e = base.clone();
                e.set_password(&self.password);
                for f in &self.fields {
                    e.set_field(&f.key, &f.value);
                }
                e.serialize()
            }
            None => {
                let mut out = String::new();
                out.push_str(&self.password);
                out.push('\n');
                for f in &self.fields {
                    if !f.value.is_empty() {
                        out.push_str(&format!("{}: {}\n", f.key, f.value));
                    }
                }
                out
            }
        }
    }
}

/// Best-effort list of `key:` field names present in an entry, in order.
/// Parses lines 1.. looking for `key: value` patterns (same heuristic used by
/// `Entry::field`). Skips blank and free-form lines (no colon, or empty key).
fn entry_field_keys(entry: &Entry) -> Vec<String> {
    entry
        .serialize()
        .lines()
        .skip(1)
        .filter_map(|line| line.split_once(':').map(|(k, _)| k.trim().to_string()))
        .filter(|k| !k.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::Entry;

    #[test]
    fn form_from_entry_exposes_password_and_fields() {
        let e = Entry::parse("pw\nuser: bob\nurl: a.com\n");
        let f = Form::from_entry("web/x", &e);
        assert_eq!(f.password, "pw");
        assert_eq!(f.field_value("user"), Some("bob"));
        assert_eq!(f.field_value("url"), Some("a.com"));
    }

    #[test]
    fn editing_a_field_then_to_contents_preserves_unknown_lines() {
        let e = Entry::parse("pw\nuser: bob\nrandom note\nurl: a.com\n");
        let mut f = Form::from_entry("web/x", &e);
        f.set_field("user", "alice");
        // round-trip: "random note" (unrecognized) must survive in place
        assert_eq!(
            f.to_contents(),
            "pw\nuser: alice\nrandom note\nurl: a.com\n"
        );
    }

    #[test]
    fn editing_password_replaces_only_first_line() {
        let e = Entry::parse("oldpw\nuser: bob\n");
        let mut f = Form::from_entry("web/x", &e);
        f.password = "newpw".to_string();
        assert_eq!(f.to_contents(), "newpw\nuser: bob\n");
    }

    #[test]
    fn new_from_template_seeds_empty_fields() {
        let f = Form::new_from_template("web/new", Template::Login);
        assert_eq!(f.password, "");
        assert_eq!(f.field_value("user"), Some(""));
        assert_eq!(f.field_value("url"), Some(""));
    }

    #[test]
    fn new_form_to_contents_emits_password_then_filled_fields() {
        let mut f = Form::new_from_template("web/new", Template::Login);
        f.password = "pw".to_string();
        f.set_field("user", "bob");
        // empty fields (url) are omitted from output
        assert_eq!(f.to_contents(), "pw\nuser: bob\n");
    }
}
