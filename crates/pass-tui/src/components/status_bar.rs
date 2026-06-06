//! StatusBar component — hint / status line shown at the bottom of the screen.
//!
//! In Phase 1 it shows the static browse-mode hint line.
//! It holds the global `q` / Ctrl-C subscription target so the quit event is
//! routed here when the user is not explicitly focused on any component.

use tui_realm_stdlib::components::Paragraph;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, HorizontalAlignment, QueryResult, Style, TextModifiers,
};
use tuirealm::ratatui::text::{Line, Span};
use tuirealm::state::State;

use crate::msg::Msg;
use crate::theme;

/// Hint/status bar shown at the bottom of the window.
pub struct StatusBar {
    inner: Paragraph,
}

impl Default for StatusBar {
    fn default() -> Self {
        // Phase-1 hint line: "q quit"
        let hint_line = Line::from(vec![
            Span::styled(
                "q",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" quit", Style::default().fg(theme::MUTED_BRIGHT)),
        ]);

        let inner = Paragraph::default()
            .background(theme::SURFACE)
            .foreground(theme::MUTED)
            .alignment_horizontal(HorizontalAlignment::Left)
            .text(vec![hint_line]);

        Self { inner }
    }
}

impl Component for StatusBar {
    fn view(
        &mut self,
        frame: &mut tuirealm::ratatui::Frame,
        area: tuirealm::ratatui::layout::Rect,
    ) {
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

impl AppComponent<Msg, NoUserEvent> for StatusBar {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::Quit),
            _ => Some(Msg::None),
        }
    }
}
