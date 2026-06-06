//! Delete-confirmation modal — a centred yes/no prompt.
//!
//! Uses a `Paragraph` to show the confirmation question and interprets
//! `y` / `n` / Enter / Esc directly, without a separate `Radio` component.
//!
//! Keys:
//!   y / Enter  → `ConfirmDelete(true)`
//!   n / Esc    → `ConfirmDelete(false)` / `CloseOverlay`

use tui_realm_stdlib::components::Paragraph;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, LineStatic, QueryResult,
    SpanStatic, Style, TextModifiers, Title,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::state::State;

use crate::msg::Msg;
use crate::theme;

pub struct ConfirmModal {
    inner: Paragraph,
    /// The entry path being deleted (kept for tests and future use).
    #[allow(dead_code)]
    pub path: String,
}

impl ConfirmModal {
    pub fn new(path: &str) -> Self {
        let lines = vec![
            LineStatic::from(vec![SpanStatic::styled(
                format!("Delete  {path}?"),
                Style::default()
                    .fg(theme::COCHINEAL)
                    .add_modifier(TextModifiers::BOLD),
            )]),
            LineStatic::from(vec![SpanStatic::raw("")]),
            LineStatic::from(vec![
                SpanStatic::styled(
                    "  y",
                    Style::default()
                        .fg(theme::GOLD)
                        .add_modifier(TextModifiers::BOLD),
                ),
                SpanStatic::styled(" / Enter", Style::default().fg(theme::MUTED_BRIGHT)),
                SpanStatic::styled("  confirm    ", Style::default().fg(theme::MUTED)),
                SpanStatic::styled(
                    "n",
                    Style::default()
                        .fg(theme::GOLD)
                        .add_modifier(TextModifiers::BOLD),
                ),
                SpanStatic::styled(" / Esc", Style::default().fg(theme::MUTED_BRIGHT)),
                SpanStatic::styled("  cancel", Style::default().fg(theme::MUTED)),
            ]),
        ];

        let inner = Paragraph::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::COCHINEAL)
                    .modifiers(BorderType::Rounded),
            )
            .alignment_horizontal(HorizontalAlignment::Left)
            .title(Title::from(" Delete Entry ").alignment(HorizontalAlignment::Center))
            .text(lines);

        Self {
            inner,
            path: path.to_string(),
        }
    }
}

impl Component for ConfirmModal {
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
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.inner.perform(cmd)
    }
}

impl AppComponent<Msg, NoUserEvent> for ConfirmModal {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            // Confirm
            Event::Keyboard(KeyEvent {
                code: Key::Char('y'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::ConfirmDelete(true)),

            // Cancel
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::ConfirmDelete(false)),

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
    fn y_key_emits_confirm_true() {
        let mut comp = ConfirmModal::new("web/github.com");
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('y'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::ConfirmDelete(true)));
    }

    #[test]
    fn enter_key_emits_confirm_true() {
        let mut comp = ConfirmModal::new("web/github.com");
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::ConfirmDelete(true)));
    }

    #[test]
    fn n_key_emits_confirm_false() {
        let mut comp = ConfirmModal::new("web/github.com");
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('n'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::ConfirmDelete(false)));
    }

    #[test]
    fn esc_key_emits_confirm_false() {
        let mut comp = ConfirmModal::new("web/github.com");
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::ConfirmDelete(false)));
    }

    #[test]
    fn confirm_stores_path() {
        let comp = ConfirmModal::new("email/work");
        assert_eq!(comp.path, "email/work");
    }
}
