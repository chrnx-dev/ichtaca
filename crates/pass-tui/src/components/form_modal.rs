//! Create / Edit entry form modal.
//!
//! Fields (in focus order):
//!   0  Entry path (only editable in Create; shown as read-only label in Edit)
//!   1  Password (masked, with Ctrl-g generate shortcut)
//!   2…n+1 Value inputs for each key/value pair row
//!          Keys are rendered as fixed muted labels — NOT editable inputs.
//!          Focus chain skips key labels; only value inputs are focusable.
//!   n+2 OTP URI
//!   n+3 Tags (space-separated)
//!
//! All editable rows use styled `tui_realm_stdlib::Input` widgets.
//! Key labels use a simple ratatui `Paragraph` rendered inline (not a mounted
//! component — they are drawn directly in `render_frame` alongside their value
//! input).  Focus is managed by the `Model` — Tab/↑/↓ emit
//! `FormFocusNext/Prev`.
//! Enter on any focused field emits `SubmitForm`.
//! Ctrl-g in the password field emits `Generate`.
//! Esc emits `CloseOverlay`.
//!
//! Because tui-realm fields are individual mounted components, this module
//! defines the **input component wrappers** while `Model` mounts/unmounts them
//! and manages the focus chain.  The `FormState` struct lives in `model.rs`
//! and carries the actual field values extracted for saving.
//!
//! ## Field-key labels (Fix 2)
//! Field keys come from the chosen template (Create) or from the existing
//! entry (Edit).  They are display-only labels — the user cannot edit them.
//! The keys are stored in `FormState.fields` as `(key_string, value_string)`
//! pairs; `collect_form_values` reads the key from `FormState` directly and
//! only reads the *value* from the mounted widget.
//!
//! Empty value → field is written with empty value (`set_field(k, "")`).
//! Auto-removal of empty-value fields is NOT done; this keeps the logic simple.
//!
//! // TODO: optional add-custom-field affordance

use tui_realm_stdlib::components::Input as TuiInput;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, InputType, QueryResult, Style,
    Title,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::state::{State, StateValue};

use crate::msg::Msg;
use crate::theme;

// ── FormMode ──────────────────────────────────────────────────────────────────

/// Whether the form is being used for creating a new entry or editing an
/// existing one.  This controls which fields are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit,
}

// ── FormField ─────────────────────────────────────────────────────────────────

/// A single labelled input field inside the form.
///
/// For the password field, `masked` should be `true`; all others plain text.
/// For the path field in Create mode, `is_path` should be `true` so that Tab
/// emits `PathTabComplete` instead of `FormFocusNext`, enabling folder
/// autocomplete (Fix 3).
pub struct FormField {
    inner: TuiInput,
    /// The semantic label (shown in the title).
    #[allow(dead_code)]
    pub label: String,
    /// Whether this field is the password field (masked + Ctrl-g).
    pub is_password: bool,
    /// Whether this is the path field (Tab → PathTabComplete in Create mode).
    pub is_path: bool,
    /// Whether the password is currently revealed (kept for tests and future use).
    #[allow(dead_code)]
    pub revealed: bool,
}

impl FormField {
    pub fn new(label: &str, initial: &str, is_password: bool) -> Self {
        let itype = if is_password {
            // always masked initially; revealed is a render toggle
            InputType::Password('•')
        } else {
            InputType::Text
        };
        // Active (focused) border: gold.  Inactive: muted.
        // tui-realm uses `.borders(…)` for the focused border colour and
        // `.inactive(…)` for the unfocused style (entire widget dimmed).
        let inner = TuiInput::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::GOLD)
                    .modifiers(BorderType::Rounded),
            )
            .inactive(Style::default().fg(theme::MUTED).bg(theme::SURFACE))
            .title(Title::from(format!(" {label} ")).alignment(HorizontalAlignment::Left))
            .input_type(itype)
            .value(initial);

        Self {
            inner,
            label: label.to_string(),
            is_password,
            is_path: false,
            revealed: false,
        }
    }

    /// Mark this field as the path field (Tab → `PathTabComplete`).
    pub fn with_path(mut self) -> Self {
        self.is_path = true;
        self
    }

    /// Toggle password visibility (only meaningful when `is_password == true`).
    #[allow(dead_code)]
    pub fn toggle_reveal(&mut self) {
        self.revealed = !self.revealed;
        let itype = if self.revealed {
            InputType::Text
        } else {
            InputType::Password('•')
        };
        self.inner
            .attr(Attribute::InputType, AttrValue::InputType(itype));
    }

    /// Read the current value as a `String`.
    #[allow(dead_code)]
    pub fn get_value(&self) -> String {
        match self.inner.state() {
            State::Single(StateValue::String(s)) => s,
            _ => String::new(),
        }
    }
}

impl Component for FormField {
    fn view(&mut self, frame: &mut tuirealm::ratatui::Frame, area: Rect) {
        self.inner.view(frame, area);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        self.inner.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.inner.attr(attr, value);
    }

    fn state(&self) -> State {
        self.inner.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.inner.perform(cmd)
    }
}

impl AppComponent<Msg, NoUserEvent> for FormField {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::SubmitForm),

            // Ctrl-s: save from any field (consistent with the Notes textarea).
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::SubmitForm),

            // Tab — path field: attempt folder autocomplete first (Fix 3).
            //        All other fields: navigate to next field.
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.is_path {
                    Some(Msg::PathTabComplete)
                } else {
                    Some(Msg::FormFocusNext)
                }
            }

            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::FormFocusPrev),

            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusNext),

            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusPrev),

            // Ctrl-g inside the password field → generate a password
            Event::Keyboard(KeyEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::CONTROL,
            }) if self.is_password => Some(Msg::Generate),

            // Normal character input — accept plain chars AND Shift-chars
            // (uppercase letters and symbols such as !@# arrive with SHIFT set).
            // Any other modifier combo (CTRL, ALT, etc.) is NOT treated as text.
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers,
            }) if modifiers.is_empty() || *modifiers == KeyModifiers::SHIFT => {
                self.perform(Cmd::Type(*ch));
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Delete);
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Cancel);
                Some(Msg::None)
            }

            Event::Tick => Some(Msg::Tick),

            _ => Some(Msg::None),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::event::{Key, KeyEvent, KeyModifiers};

    #[test]
    fn form_field_esc_emits_close_overlay() {
        let mut f = FormField::new("test", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }

    #[test]
    fn form_field_enter_emits_submit() {
        let mut f = FormField::new("test", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::SubmitForm));
    }

    #[test]
    fn form_field_tab_emits_focus_next() {
        let mut f = FormField::new("test", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Tab,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::FormFocusNext));
    }

    #[test]
    fn form_field_password_ctrl_g_emits_generate() {
        let mut f = FormField::new("password", "", true);
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            modifiers: KeyModifiers::CONTROL,
        }));
        assert_eq!(msg, Some(Msg::Generate));
    }

    #[test]
    fn non_password_field_ctrl_g_types_g() {
        let mut f = FormField::new("user", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            modifiers: KeyModifiers::CONTROL,
        }));
        // Not Generate (not a password field)
        assert_ne!(msg, Some(Msg::Generate));
    }

    #[test]
    fn form_field_typing_updates_value() {
        let mut f = FormField::new("user", "", false);
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('l'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('i'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('c'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('e'),
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_value(), "alice");
    }

    #[test]
    fn form_field_initial_value_readable() {
        let f = FormField::new("url", "example.com", false);
        assert_eq!(f.get_value(), "example.com");
    }

    #[test]
    fn password_field_masked_by_default() {
        let f = FormField::new("password", "s3cr3t", true);
        // The InputType should be Password — query and check via into_attr
        if let Some(qr) = f.query(Attribute::InputType) {
            let av = qr.into_attr();
            if let AttrValue::InputType(itype) = av {
                assert!(
                    matches!(itype, InputType::Password(_)),
                    "password field must use Password InputType"
                );
            }
        }
        // But the actual value is still readable (unmasked)
        assert_eq!(f.get_value(), "s3cr3t");
    }

    #[test]
    fn toggle_reveal_switches_input_type() {
        let mut f = FormField::new("password", "s3cr3t", true);
        assert!(!f.revealed);
        f.toggle_reveal();
        assert!(f.revealed);
        if let Some(qr) = f.query(Attribute::InputType) {
            let av = qr.into_attr();
            if let AttrValue::InputType(itype) = av {
                assert!(
                    matches!(itype, InputType::Text),
                    "after reveal, InputType must be Text"
                );
            }
        }
        f.toggle_reveal();
        assert!(!f.revealed);
    }

    // ── Fix 1: Shift modifier accepted for uppercase/symbol input ────────────

    /// Shift+letter must type the uppercase character into the input.
    #[test]
    fn shift_char_types_uppercase() {
        let mut f = FormField::new("user", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('A'),
            modifiers: KeyModifiers::SHIFT,
        }));
        // Should return None (consumed, not a nav message) and the value updated.
        assert_eq!(msg, Some(Msg::None), "Shift+A must be consumed as text");
        assert_eq!(f.get_value(), "A", "Shift+A must produce uppercase 'A'");
    }

    /// Plain lowercase must still work (regression guard).
    #[test]
    fn plain_char_still_types() {
        let mut f = FormField::new("user", "", false);
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_value(), "a", "plain 'a' must still type");
    }

    /// Ctrl-g on a password field must still emit Generate, not be treated as text.
    #[test]
    fn ctrl_g_on_password_still_generates() {
        let mut f = FormField::new("password", "", true);
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            modifiers: KeyModifiers::CONTROL,
        }));
        assert_eq!(
            msg,
            Some(Msg::Generate),
            "Ctrl-g on password field must still emit Generate"
        );
        // Value must not have 'g' typed in
        assert_eq!(
            f.get_value(),
            "",
            "Generate must not type a character into the field"
        );
    }

    /// Ctrl-g on a NON-password field: NOT Generate AND NOT typed as text.
    #[test]
    fn ctrl_g_on_non_password_not_generate_not_text() {
        let mut f = FormField::new("user", "", false);
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            modifiers: KeyModifiers::CONTROL,
        }));
        assert_ne!(msg, Some(Msg::Generate), "must not be Generate");
        // The default _ arm returns None without typing.
        assert_eq!(
            f.get_value(),
            "",
            "Ctrl-g on non-password must not type a character"
        );
    }
}
