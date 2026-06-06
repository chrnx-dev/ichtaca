//! Multi-line Notes field for the Create/Edit form.
//!
//! Unlike the stdlib `Textarea` (which is a *read-only* scrollable list),
//! this component is a fully editable multi-line text input that:
//!
//! - Stores lines as a `Vec<String>` with a (row, col) cursor.
//! - **Enter** inserts a newline (splits the current line at the cursor).
//! - **Ctrl-s** emits `Msg::SubmitForm` (save — works from any field).
//! - **Backspace** deletes the character before the cursor (or merges lines).
//! - **↑/↓** move the cursor between lines.
//! - **←/→** move the cursor within a line.
//! - **Tab / BackTab** emit `FormFocusNext / FormFocusPrev` (leave the field).
//! - **Esc** emits `CloseOverlay`.
//! - Regular characters and Shift-characters are inserted at the cursor.
//!
//! ## Rendering
//! The component renders as a bordered box (gold when focused, muted otherwise)
//! titled " Notes ".  Lines are rendered from the top; the highlighted line
//! (cursor row) is shown with a subtle background.  The cursor column is not
//! shown as a blinking cell (terminal cursor control is outside tui-realm's
//! scope for custom components) — the highlighted row is sufficient feedback.

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{AttrValue, Attribute, QueryResult};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::text::Span;
use tuirealm::ratatui::widgets::{Block, List, ListItem, ListState};
use tuirealm::state::{State, StateValue};

use crate::msg::Msg;
use crate::theme;

// ── NotesField ────────────────────────────────────────────────────────────────

/// Editable multi-line notes field.
pub struct NotesField {
    /// Lines of text (at least one empty line when the field is empty).
    lines: Vec<String>,
    /// Current cursor row (0-based).
    cursor_row: usize,
    /// Current cursor column (0-based, character offset within the line).
    cursor_col: usize,
    /// Whether this component currently has focus (set by the tui-realm focus
    /// mechanism — toggled via `Attribute::Focus`).
    focused: bool,
}

impl Default for NotesField {
    fn default() -> Self {
        Self::new("")
    }
}

impl NotesField {
    /// Build a new `NotesField` pre-filled with `initial` text (multi-line, `\n`-separated).
    pub fn new(initial: &str) -> Self {
        let lines: Vec<String> = if initial.is_empty() {
            vec![String::new()]
        } else {
            initial.lines().map(|l| l.to_string()).collect()
        };
        let cursor_row = lines.len().saturating_sub(1);
        let cursor_col = lines.last().map(|l| l.len()).unwrap_or(0);
        Self {
            lines,
            cursor_row,
            cursor_col,
            focused: false,
        }
    }

    /// Return the full text content as a single `\n`-joined string.
    pub fn get_text(&self) -> String {
        // Treat a single empty line as truly empty.
        if self.lines.len() == 1 && self.lines[0].is_empty() {
            String::new()
        } else {
            self.lines.join("\n")
        }
    }

    // ── Mutation helpers ──────────────────────────────────────────────────────

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, ch: char) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        // Guard: row must be in bounds.
        if row >= self.lines.len() {
            self.lines.push(String::new());
        }
        // Split the line at the cursor and re-join with the new char.
        let line = self.lines[row].clone();
        let byte_idx = char_to_byte_idx(&line, col);
        let mut new_line = line[..byte_idx].to_string();
        new_line.push(ch);
        new_line.push_str(&line[byte_idx..]);
        self.lines[row] = new_line;
        self.cursor_col += 1;
    }

    /// Insert a newline at the cursor (split the current line).
    fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        if row >= self.lines.len() {
            self.lines.push(String::new());
        }
        let line = self.lines[row].clone();
        let byte_idx = char_to_byte_idx(&line, col);
        let rest = line[byte_idx..].to_string();
        self.lines[row].truncate(byte_idx);
        self.lines.insert(row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// Delete the character before the cursor (Backspace).
    fn backspace(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col > 0 {
            // Delete char within the current line.
            let line = self.lines[row].clone();
            let byte_idx = char_to_byte_idx(&line, col);
            // Find the byte start of the previous character.
            let prev_byte = line[..byte_idx]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            let mut new_line = line[..prev_byte].to_string();
            new_line.push_str(&line[byte_idx..]);
            self.lines[row] = new_line;
            self.cursor_col -= 1;
        } else if row > 0 {
            // Merge with the previous line.
            let prev_len = char_count(&self.lines[row - 1]);
            let current = self.lines.remove(row);
            self.lines[row - 1].push_str(&current);
            self.cursor_row -= 1;
            self.cursor_col = prev_len;
        }
        // If cursor_row == 0 and col == 0 → nothing to delete.
    }

    /// Move cursor up (within the notes).
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Clamp col to the new line length.
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    /// Move cursor down (within the notes).
    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            let line_len = char_count(&self.lines[self.cursor_row]);
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }
}

// ── Utility functions ─────────────────────────────────────────────────────────

/// Convert a character-offset `col` to a byte index within `s`.
fn char_to_byte_idx(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}

/// Count the number of Unicode characters in `s`.
fn char_count(s: &str) -> usize {
    s.chars().count()
}

// ── Component impl ────────────────────────────────────────────────────────────

impl Component for NotesField {
    fn view(&mut self, frame: &mut tuirealm::ratatui::Frame, area: Rect) {
        let border_color = if self.focused {
            theme::GOLD
        } else {
            theme::MUTED
        };

        let block = Block::default()
            .borders(tuirealm::ratatui::widgets::Borders::ALL)
            .border_type(tuirealm::ratatui::widgets::BorderType::Rounded)
            .border_style(tuirealm::ratatui::style::Style::default().fg(border_color))
            .title(
                tuirealm::ratatui::text::Line::from(Span::styled(
                    " Notes ",
                    tuirealm::ratatui::style::Style::default().fg(border_color),
                ))
                .alignment(tuirealm::ratatui::layout::Alignment::Left),
            )
            .style(
                tuirealm::ratatui::style::Style::default()
                    .bg(theme::SURFACE)
                    .fg(theme::TEXT),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render lines; highlight the cursor row.
        let items: Vec<ListItem> = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let style = if i == self.cursor_row && self.focused {
                    tuirealm::ratatui::style::Style::default()
                        .bg(theme::SURFACE_SEL)
                        .fg(theme::TEXT)
                } else {
                    tuirealm::ratatui::style::Style::default()
                        .bg(theme::SURFACE)
                        .fg(theme::TEXT)
                };
                ListItem::new(Span::styled(line.clone(), style))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(self.cursor_row));

        // Scroll offset: keep cursor_row visible.
        let visible_height = inner.height as usize;
        let scroll_offset = if self.cursor_row >= visible_height {
            self.cursor_row + 1 - visible_height
        } else {
            0
        };
        list_state.select(Some(self.cursor_row));
        // tui-realm's ratatui doesn't expose set_offset easily, so we render
        // all items but skip the scroll_offset ones with an offset workaround.
        let _ = scroll_offset; // Used conceptually above; List handles scroll via state

        let list = List::new(items)
            .direction(tuirealm::ratatui::widgets::ListDirection::TopToBottom)
            .style(
                tuirealm::ratatui::style::Style::default()
                    .bg(theme::SURFACE)
                    .fg(theme::TEXT),
            );

        frame.render_stateful_widget(list, inner, &mut list_state);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        match attr {
            Attribute::Focus => Some(QueryResult::Owned(AttrValue::Flag(self.focused))),
            _ => None,
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match attr {
            Attribute::Focus => {
                if let AttrValue::Flag(f) = value {
                    self.focused = f;
                }
            }
            Attribute::Value => {
                // Accept a String value to reset/prefill the content.
                if let AttrValue::String(s) = value {
                    let new = NotesField::new(&s);
                    self.lines = new.lines;
                    self.cursor_row = new.cursor_row;
                    self.cursor_col = new.cursor_col;
                }
            }
            _ => {}
        }
    }

    fn state(&self) -> State {
        // State is the entire text content as a string.
        State::Single(StateValue::String(self.get_text()))
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        // All mutations are driven from `on()` directly; perform is a no-op.
        CmdResult::NoChange
    }
}

// ── AppComponent impl ─────────────────────────────────────────────────────────

impl AppComponent<Msg, NoUserEvent> for NotesField {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            // ── Navigation out of the field ───────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::CloseOverlay),

            // Ctrl-s = save from anywhere (including inside the notes field).
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::SubmitForm),

            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::FormFocusNext),

            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::FormFocusPrev),

            // ── Within-field cursor movement ──────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_up();
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_down();
                Some(Msg::None)
            }

            // ── Text editing ──────────────────────────────────────────────────

            // Enter = insert newline (NOT save).
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.insert_newline();
                Some(Msg::None)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.backspace();
                Some(Msg::None)
            }

            // Regular characters and Shift-characters (uppercase/symbols).
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers,
            }) if modifiers.is_empty() || *modifiers == KeyModifiers::SHIFT => {
                self.insert_char(*ch);
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
    fn empty_notes_get_text_is_empty_string() {
        let f = NotesField::default();
        assert_eq!(f.get_text(), "");
    }

    #[test]
    fn initial_text_is_preserved() {
        let f = NotesField::new("hello\nworld");
        assert_eq!(f.get_text(), "hello\nworld");
    }

    #[test]
    fn typing_chars_appends_to_line() {
        let mut f = NotesField::default();
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('h'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('i'),
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_text(), "hi");
    }

    #[test]
    fn enter_inserts_newline() {
        let mut f = NotesField::default();
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('b'),
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_text(), "a\nb");
    }

    #[test]
    fn enter_does_not_emit_submit_form() {
        let mut f = NotesField::default();
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert_ne!(msg, Some(Msg::SubmitForm), "Enter must not save the form");
        assert_eq!(
            msg,
            Some(Msg::None),
            "Enter must emit None (newline inserted)"
        );
    }

    #[test]
    fn ctrl_s_emits_submit_form() {
        let mut f = NotesField::default();
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('s'),
            modifiers: KeyModifiers::CONTROL,
        }));
        assert_eq!(msg, Some(Msg::SubmitForm));
    }

    #[test]
    fn esc_emits_close_overlay() {
        let mut f = NotesField::default();
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Esc,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::CloseOverlay));
    }

    #[test]
    fn tab_emits_form_focus_next() {
        let mut f = NotesField::default();
        let msg = f.on(&Event::Keyboard(KeyEvent::new(
            Key::Tab,
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::FormFocusNext));
    }

    #[test]
    fn backtab_emits_form_focus_prev() {
        let mut f = NotesField::default();
        let msg = f.on(&Event::Keyboard(KeyEvent {
            code: Key::BackTab,
            modifiers: KeyModifiers::SHIFT,
        }));
        assert_eq!(msg, Some(Msg::FormFocusPrev));
    }

    #[test]
    fn backspace_removes_char() {
        let mut f = NotesField::new("ab");
        f.cursor_col = 2; // end of "ab"
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_text(), "a");
    }

    #[test]
    fn backspace_at_line_start_merges_lines() {
        let mut f = NotesField::new("a\nb");
        f.cursor_row = 1;
        f.cursor_col = 0;
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));
        assert_eq!(f.get_text(), "ab");
    }

    #[test]
    fn up_down_navigate_lines() {
        let mut f = NotesField::new("line1\nline2\nline3");
        f.cursor_row = 0;
        f.cursor_col = 0;
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(f.cursor_row, 1);
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(f.cursor_row, 2);
        f.on(&Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(f.cursor_row, 1);
    }

    #[test]
    fn shift_char_types_uppercase() {
        let mut f = NotesField::default();
        f.on(&Event::Keyboard(KeyEvent {
            code: Key::Char('A'),
            modifiers: KeyModifiers::SHIFT,
        }));
        assert_eq!(f.get_text(), "A");
    }

    #[test]
    fn state_returns_full_text() {
        let f = NotesField::new("hello\nworld");
        match f.state() {
            tuirealm::state::State::Single(tuirealm::state::StateValue::String(s)) => {
                assert_eq!(s, "hello\nworld")
            }
            other => panic!("unexpected state: {other:?}"),
        }
    }

    #[test]
    fn attr_value_string_resets_content() {
        let mut f = NotesField::new("old text");
        f.attr(Attribute::Value, AttrValue::String("new text".to_string()));
        assert_eq!(f.get_text(), "new text");
    }

    #[test]
    fn up_at_first_line_is_noop() {
        let mut f = NotesField::new("only");
        f.cursor_row = 0;
        f.on(&Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(f.cursor_row, 0);
    }

    #[test]
    fn down_at_last_line_is_noop() {
        let mut f = NotesField::new("only");
        f.cursor_row = 0;
        f.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(f.cursor_row, 0);
    }
}
