//! Bottom status/help bar: keybinding hints + transient notifications.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{AppState, Mode, NoticeKind};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (text, style) = if let Some(n) = &state.notification {
        let color = match n.kind {
            NoticeKind::Info => Color::Green,
            NoticeKind::Error => Color::Red,
        };
        (n.text.clone(), Style::default().fg(color))
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
        (hint.to_string(), Style::default())
    };
    frame.render_widget(Paragraph::new(text).style(style), area);
}
