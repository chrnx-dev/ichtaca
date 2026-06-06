//! Bottom status/help bar: keybinding hints + transient notifications.
//!
//! Hints render each key in gold (bold) and its label in a readable tone so the
//! bar is legible against the obsidian background — not a flat muted line.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{AppState, Mode, NoticeKind};
use crate::theme;

/// (key, label) hint pairs per mode.
fn hints(mode: &Mode) -> &'static [(&'static str, &'static str)] {
    match mode {
        Mode::Browse => &[
            ("↑↓/jk", "move"),
            ("←→/hl", "fold"),
            ("/", "search"),
            ("c", "copy"),
            ("s", "reveal"),
            ("a", "add"),
            ("e", "edit"),
            ("d", "del"),
            ("q", "quit"),
        ],
        Mode::Search => &[
            ("type", "filter"),
            ("↑↓", "move"),
            ("Enter", "select"),
            ("Esc", "cancel"),
        ],
        Mode::EditForm => &[
            ("type", "edit"),
            ("Tab/↑↓", "fields"),
            ("^g", "generate"),
            ("Enter", "save"),
            ("Esc", "cancel"),
        ],
        Mode::Confirm(_) => &[("y", "confirm"), ("n", "cancel")],
        Mode::Help => &[("q", "quit")],
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    // A transient notification takes over the bar when present.
    if let Some(n) = &state.notification {
        let style = match n.kind {
            NoticeKind::Info => theme::success(),
            NoticeKind::Error => theme::error(),
        };
        frame.render_widget(Paragraph::new(n.text.clone()).style(style), area);
        return;
    }

    // Otherwise render the keybinding hints with highlighted keys.
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label)) in hints(&state.mode).iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", theme::hint_label()));
        }
        spans.push(Span::styled(*key, theme::hint_key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, theme::hint_label()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
