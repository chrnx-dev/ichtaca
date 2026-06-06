//! Bottom status/help bar: keybinding hints + transient notifications.

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{AppState, Mode, NoticeKind};
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (text, style) = if let Some(n) = &state.notification {
        let s = match n.kind {
            NoticeKind::Info => theme::success(),
            NoticeKind::Error => theme::error(),
        };
        (n.text.clone(), s)
    } else {
        let hint = match state.mode {
            Mode::Browse => {
                "j/k move  h/l fold  / search  c copy  s reveal  a add  e edit  d del  q quit"
            }
            Mode::Search => "type to filter  ↑↓ move  Enter select  Esc cancel",
            Mode::EditForm => {
                "type to edit  Tab/↑↓ fields  Ctrl-g generate  Enter save  Esc cancel"
            }
            Mode::Confirm(_) => "y confirm  n cancel",
            Mode::Help => "q quit",
        };
        (hint.to_string(), theme::hint())
    };
    frame.render_widget(Paragraph::new(text).style(style), area);
}
