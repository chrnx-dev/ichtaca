//! Confirm modal + help/startup screen.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::Confirm;
use crate::theme;

pub fn render_confirm(frame: &mut Frame, area: Rect, confirm: &Confirm) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("Confirm", theme::title()));
    // Render the prompt in danger/cochineal colour to signal this is destructive.
    let prompt = Line::from(Span::styled(confirm.prompt.clone(), theme::error()));
    frame.render_widget(Paragraph::new(prompt).block(block), area);
}

pub fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("pass-tui \u{2014} setup", theme::title()));
    let body = "pass / gpg / store not found.\n\n\
        Install pass and gpg, then run `pass init <gpg-id>`.\n\n\
        Press q to quit.";
    frame.render_widget(Paragraph::new(body).block(block), area);
}
