//! Create/edit form overlay.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let title = if state.form.as_ref().map(|f| f.editing).unwrap_or(false) {
        "Edit entry"
    } else {
        "New entry"
    };
    let mut lines: Vec<Line> = Vec::new();
    if let Some(f) = &state.form {
        lines.push(Line::from(vec![
            Span::styled("path", theme::key()),
            Span::styled(": ", theme::key()),
            Span::styled(f.path.clone(), theme::text()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("password", theme::key()),
            Span::styled(": ", theme::key()),
            Span::styled(f.password.clone(), theme::text()),
        ]));
        for field in &f.fields {
            lines.push(Line::from(vec![
                Span::styled(field.key.clone(), theme::key()),
                Span::styled(": ", theme::key()),
                Span::styled(field.value.clone(), theme::text()),
            ]));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(title, theme::title()));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
