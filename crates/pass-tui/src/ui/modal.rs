//! Confirm modal + help/startup screen.

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::Confirm;

pub fn render_confirm(frame: &mut Frame, area: Rect, confirm: &Confirm) {
    frame.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title("Confirm");
    frame.render_widget(Paragraph::new(confirm.prompt.clone()).block(block), area);
}

pub fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("pass-tui \u{2014} setup");
    let body = "pass / gpg / store not found.\n\n\
        Install pass and gpg, then run `pass init <gpg-id>`.\n\n\
        Press q to quit.";
    frame.render_widget(Paragraph::new(body).block(block), area);
}
