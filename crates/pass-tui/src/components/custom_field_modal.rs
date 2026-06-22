//! Add-custom-field prompt components.

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

/// Single-line input used by the add-custom-field prompt.
pub struct CustomFieldInput {
    inner: TuiInput,
}

impl CustomFieldInput {
    pub fn new(label: &str, initial: &str) -> Self {
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
            .input_type(InputType::Text)
            .value(initial);

        Self { inner }
    }

    #[allow(dead_code)]
    pub fn get_value(&self) -> String {
        match self.inner.state() {
            State::Single(StateValue::String(s)) => s,
            _ => String::new(),
        }
    }
}

impl Component for CustomFieldInput {
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

impl AppComponent<Msg, NoUserEvent> for CustomFieldInput {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::SubmitCustomField),

            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CustomFieldFocusNext),

            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::CustomFieldFocusPrev),

            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::SubmitCustomField),

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

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::component::AppComponent;
    use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};

    #[test]
    fn custom_field_input_esc_closes_overlay() {
        let mut input = CustomFieldInput::new("Key", "");
        let msg = input.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(crate::msg::Msg::CloseOverlay));
    }

    #[test]
    fn custom_field_input_enter_submits() {
        let mut input = CustomFieldInput::new("Key", "");
        let msg = input.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(crate::msg::Msg::SubmitCustomField));
    }

    #[test]
    fn custom_field_input_tab_moves_next() {
        let mut input = CustomFieldInput::new("Key", "");
        let msg = input.on(&Event::Keyboard(KeyEvent::new(
            Key::Tab,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(crate::msg::Msg::CustomFieldFocusNext));
    }

    #[test]
    fn custom_field_input_backtab_moves_prev() {
        let mut input = CustomFieldInput::new("Value", "");
        let msg = input.on(&Event::Keyboard(KeyEvent::new(
            Key::BackTab,
            KeyModifiers::SHIFT,
        )));
        assert_eq!(msg, Some(crate::msg::Msg::CustomFieldFocusPrev));
    }

    #[test]
    fn custom_field_input_typing_updates_value() {
        let mut input = CustomFieldInput::new("Key", "");
        input.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::NONE,
        )));
        input.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('c'),
            KeyModifiers::NONE,
        )));
        assert_eq!(input.get_value(), "ac");
    }
}
