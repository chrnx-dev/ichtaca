//! Create / Edit entry form modal.
//!
//! Fields (in focus order):
//!   0  Template selector (only for Create; hidden for Edit)
//!   1  Entry path (only for Create; shown but uneditable for Edit display)
//!   2  Password (masked, with Ctrl-g generate shortcut)
//!   3…n Key/Value pair rows  (key input + value input, interleaved)
//!   n+1 OTP URI
//!   n+2 Tags (space-separated)
//!
//! All inputs are rendered as styled `tui_realm_stdlib::Input` widgets.
//! Focus is managed by the `Model` — Tab/↑/↓ emit `FormFocusNext/Prev`.
//! Enter on any focused field emits `SubmitForm`.
//! Ctrl-g in the password field emits `Generate`.
//! Esc emits `CloseOverlay`.
//!
//! Because tui-realm fields are individual mounted components, this module
//! defines the **input component wrappers** while `Model` mounts/unmounts them
//! and manages the focus chain.  The `FormState` struct lives in `model.rs`
//! and carries the actual field values extracted for saving.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMode {
    Create,
    Edit,
}

// ── FormField ─────────────────────────────────────────────────────────────────

/// A single labelled input field inside the form.
///
/// For the password field, `masked` should be `true`; all others plain text.
pub struct FormField {
    inner: TuiInput,
    /// The semantic label (shown in the title).
    #[allow(dead_code)]
    pub label: String,
    /// Whether this field is the password field (masked + Ctrl-g).
    pub is_password: bool,
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
            revealed: false,
        }
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

            // Tab / Shift-Tab — navigate between fields
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusNext),

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

            // Normal character input
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => {
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
}
