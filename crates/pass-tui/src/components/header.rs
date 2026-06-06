//! Header component — brand bar shown at the top of the screen.
//!
//! Renders "ICHTACA · lo oculto" in gold on the obsidian background.
//! Produces `Msg::None` for every event it receives (it is display-only).

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
use crate::theme::icons;

/// Header bar displaying the Ichtaca brand.
pub struct Header {
    inner: Paragraph,
}

impl Default for Header {
    fn default() -> Self {
        let brand_line = Line::from(vec![
            Span::styled(
                format!("{} ICHTACA", icons::LOCK),
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled("  ·  lo oculto", Style::default().fg(theme::MUTED)),
        ]);

        let inner = Paragraph::default()
            .background(theme::BG)
            .foreground(theme::GOLD)
            .alignment_horizontal(HorizontalAlignment::Center)
            .text(vec![brand_line]);

        Self { inner }
    }
}

impl Component for Header {
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

impl AppComponent<Msg, NoUserEvent> for Header {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        // q and Ctrl-C handled here as a backstop; the global sub on StatusBar
        // is the primary quit path, but having it here too is harmless.
        if let Event::Keyboard(KeyEvent {
            code: Key::Char('q'),
            modifiers: KeyModifiers::NONE,
        }) = ev
        {
            return Some(Msg::Quit);
        }
        if let Event::Keyboard(KeyEvent {
            code: Key::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) = ev
        {
            return Some(Msg::Quit);
        }
        Some(Msg::None)
    }
}
