//! Create/edit form overlay.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let title = if state.form.as_ref().map(|f| f.editing).unwrap_or(false) {
        "Edit entry"
    } else {
        "New entry"
    };
    let mut lines: Vec<Line> = Vec::new();
    if let Some(f) = &state.form {
        lines.push(Line::from(format!("path: {}", f.path)));
        lines.push(Line::from(format!("password: {}", f.password)));
        for field in &f.fields {
            lines.push(Line::from(format!("{}: {}", field.key, field.value)));
        }
    }
    let block = Block::default().borders(Borders::ALL).title(title);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
