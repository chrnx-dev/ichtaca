//! Template-pick modal — choose an entry template before opening the Create form.
//!
//! Shown as a small centred popup when the user presses `a` (new entry):
//!
//! ```text
//! ┌────── New Entry — Choose Template ──────┐
//! │ ▸ Login                (user, url)       │
//! │   OAuth / API   (client_id, url, …)      │
//! │   Server / SSH         (host, user, …)   │
//! │   Note                                   │
//! │   Blank                                  │
//! └─────────────────────────────────────────┘
//!   ↑↓ / jk select  ·  Enter confirm  ·  Esc cancel
//! ```
//!
//! Keys:
//!   ↑/k / ↓/j  — move selection
//!   Enter       — `SelectTemplate(idx)` (opens the form pre-filled)
//!   Esc         — `CloseOverlay`

use tui_realm_stdlib::components::List as TuiList;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, PropPayload, PropValue,
    QueryResult, Style, Title,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::state::{State, StateValue};

use crate::msg::Msg;
use crate::theme;

/// Template selection modal.
pub struct TemplateModal {
    inner: TuiList,
    /// Number of templates displayed (used in tests).
    #[allow(dead_code)]
    pub count: usize,
}

impl TemplateModal {
    /// Build a new modal from the resolved template list.
    pub fn new(templates: &[passcore::Template]) -> Self {
        use tuirealm::props::LineStatic;

        let rows: Vec<tuirealm::props::LineStatic> = templates
            .iter()
            .map(|t| {
                let hint = if t.fields.is_empty() {
                    String::new()
                } else {
                    format!("  ({})", t.fields.join(", "))
                };
                LineStatic::from(format!("{}{hint}", t.name))
            })
            .collect();

        let mut inner = TuiList::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::GOLD)
                    .modifiers(BorderType::Rounded),
            )
            .inactive(Style::default().fg(theme::MUTED).bg(theme::SURFACE))
            .highlight_style(theme::selection())
            .highlight_str("▸")
            .scroll(true)
            .rewind(true)
            .title(
                Title::from(" New Entry — Choose Template  [Enter confirm · Esc cancel] ")
                    .alignment(HorizontalAlignment::Center),
            );

        inner.attr(
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(
                rows.into_iter().map(PropValue::TextLine).collect(),
            )),
        );
        // Pre-select first row.
        inner.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::Single(PropValue::Usize(0))),
        );

        Self {
            inner,
            count: templates.len(),
        }
    }

    fn selected_idx(&self) -> usize {
        match self.inner.state() {
            State::Single(StateValue::Usize(i)) => i,
            _ => 0,
        }
    }
}

impl Component for TemplateModal {
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

impl AppComponent<Msg, NoUserEvent> for TemplateModal {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::SelectTemplate(self.selected_idx())),

            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Move(Direction::Up));
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

    fn make_modal() -> TemplateModal {
        let templates = passcore::Template::resolve(&passcore::Config::default());
        TemplateModal::new(&templates)
    }

    #[test]
    fn esc_emits_close_overlay() {
        let mut comp = make_modal();
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }

    #[test]
    fn enter_emits_select_template_with_index() {
        let mut comp = make_modal();
        // Default selection is 0 (Login)
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::SelectTemplate(0)));
    }

    #[test]
    fn down_moves_selection() {
        let mut comp = make_modal();
        comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::SelectTemplate(1)));
    }

    #[test]
    fn template_count_matches_resolved() {
        let comp = make_modal();
        // Default: Login, OAuth/API, Server/SSH, Note, Blank = 5
        assert_eq!(comp.count, 5);
    }
}
