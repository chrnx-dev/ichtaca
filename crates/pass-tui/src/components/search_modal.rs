//! Search modal — fuzzy path search displayed as a centred overlay.
//!
//! Layout (inside the popup):
//!   ┌──────────────── Search ────────────────┐
//!   │ > _query_                               │  ← `SearchInput` (Input)
//!   │──────────────────────────────────────── │
//!   │  web/github.com                         │  ← `SearchResults` (List)
//!   │  web/gitlab.com                         │
//!   └─────────────────────────────────────────┘
//!
//! Key bindings (active when the modal is focused):
//!  - Any printable char / backspace : typed into the query `Input`
//!  - ↑ / ↓            : move selection in the results `List`
//!  - Enter            : `SearchPick(selected_path)`
//!  - Esc              : `CloseOverlay`

use tui_realm_stdlib::components::{Input as TuiInput, List as TuiList};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, InputType, PropPayload,
    PropValue, QueryResult, Style, Title,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::state::{State, StateValue};

use crate::msg::Msg;
use crate::theme;

// ── SearchInput ───────────────────────────────────────────────────────────────

/// The text-input field inside the search modal.
pub struct SearchInput {
    inner: TuiInput,
}

impl Default for SearchInput {
    fn default() -> Self {
        let inner = TuiInput::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::GOLD)
                    .modifiers(BorderType::Rounded),
            )
            .inactive(Style::default().fg(theme::MUTED))
            .title(Title::from("Search").alignment(HorizontalAlignment::Left))
            .input_type(InputType::Text);

        Self { inner }
    }
}

impl Component for SearchInput {
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

impl AppComponent<Msg, NoUserEvent> for SearchInput {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            // Esc — close the modal
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

            // Typing — feed into the input and emit SearchChanged
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => {
                let res = self.perform(Cmd::Type(*ch));
                match res {
                    CmdResult::Changed(State::Single(StateValue::String(q))) => {
                        Some(Msg::SearchChanged(q))
                    }
                    _ => Some(Msg::None),
                }
            }

            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                let res = self.perform(Cmd::Delete);
                match res {
                    CmdResult::Changed(State::Single(StateValue::String(q))) => {
                        Some(Msg::SearchChanged(q))
                    }
                    CmdResult::Changed(State::None) | CmdResult::NoChange => {
                        // Input is now empty → report empty query
                        Some(Msg::SearchChanged(String::new()))
                    }
                    _ => Some(Msg::None),
                }
            }

            // ↓ / ↑ — move focus to results; we bubble FormFocusNext/Prev
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusNext),

            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusPrev),

            // Enter — pick the first/currently-selected result (no-op here;
            //         handled in SearchResults; but emit None to redraw)
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusNext),

            Event::Tick => Some(Msg::Tick),

            _ => Some(Msg::None),
        }
    }
}

// ── SearchResults ─────────────────────────────────────────────────────────────

/// The scrollable list of fuzzy-matched paths.
pub struct SearchResults {
    inner: TuiList,
    /// The current list of paths displayed (parallel to the list rows).
    pub paths: Vec<String>,
}

impl Default for SearchResults {
    fn default() -> Self {
        let inner = TuiList::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::MUTED)
                    .modifiers(BorderType::Rounded),
            )
            .inactive(Style::default().fg(theme::MUTED))
            .highlight_style(theme::selection())
            .highlight_str("▸")
            .scroll(true)
            .rewind(true);

        Self {
            inner,
            paths: Vec::new(),
        }
    }
}

impl SearchResults {
    /// Replace the displayed list with new paths.
    pub fn set_paths(&mut self, paths: Vec<String>) {
        // Build rows from paths — use owned String → Line<'static>
        let rows: Vec<tuirealm::props::LineStatic> = paths
            .iter()
            .map(|p| tuirealm::props::LineStatic::from(p.clone()))
            .collect();
        self.inner.attr(
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(
                rows.into_iter().map(PropValue::TextLine).collect(),
            )),
        );
        self.paths = paths;
        // Reset selection to top
        self.inner.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::Single(PropValue::Usize(0))),
        );
    }

    /// The currently-selected path, if any.
    pub fn selected_path(&self) -> Option<&str> {
        match self.inner.state() {
            State::Single(StateValue::Usize(idx)) => self.paths.get(idx).map(String::as_str),
            _ => self.paths.first().map(String::as_str),
        }
    }
}

impl Component for SearchResults {
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

impl AppComponent<Msg, NoUserEvent> for SearchResults {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

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
                // If already at the top, move focus back to the input
                if let State::Single(StateValue::Usize(0)) = self.inner.state() {
                    return Some(Msg::FormFocusPrev);
                }
                self.perform(Cmd::Move(Direction::Up));
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                // Emit SearchPick with the currently selected path.
                if let Some(path) = self.selected_path() {
                    Some(Msg::SearchPick(path.to_string()))
                } else {
                    Some(Msg::CloseOverlay)
                }
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
    fn search_input_esc_emits_close_overlay() {
        let mut comp = SearchInput::default();
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }

    #[test]
    fn search_input_typing_emits_search_changed() {
        let mut comp = SearchInput::default();
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('g'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::SearchChanged("g".to_string())));
    }

    #[test]
    fn search_input_backspace_emits_search_changed() {
        let mut comp = SearchInput::default();
        // Type something first
        comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('g'),
            KeyModifiers::NONE,
        )));
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));
        assert!(
            matches!(msg, Some(Msg::SearchChanged(_))),
            "backspace must emit SearchChanged"
        );
    }

    #[test]
    fn search_results_set_paths_and_pick() {
        let mut comp = SearchResults::default();
        comp.set_paths(vec!["web/a".to_string(), "web/b".to_string()]);
        // Default selection is index 0
        assert_eq!(comp.selected_path(), Some("web/a"));

        // Move down
        comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(comp.selected_path(), Some("web/b"));

        // Enter → SearchPick with current path
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::SearchPick("web/b".to_string())));
    }

    #[test]
    fn search_results_esc_closes() {
        let mut comp = SearchResults::default();
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }

    #[test]
    fn search_results_empty_enter_closes_overlay() {
        let mut comp = SearchResults::default();
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }
}
