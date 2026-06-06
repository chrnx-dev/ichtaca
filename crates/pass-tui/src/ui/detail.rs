//! Right panel: the selected entry's detail. Password hidden unless revealed.

use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::otp::OtpView;
use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title("Detail");
    let text = match &state.detail {
        None => Text::from("Select an entry"),
        Some(entry) => {
            let mut lines: Vec<Line> = Vec::new();
            let pw = if state.reveal {
                entry.password().to_string()
            } else {
                "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".to_string()
            };
            lines.push(Line::from(format!("password: {pw}")));
            // Known fields (skip line 0 = password).
            for raw in entry.serialize().lines().skip(1) {
                if raw.trim_start().starts_with("otpauth://") {
                    continue; // shown separately below
                }
                if !raw.is_empty() {
                    lines.push(Line::from(raw.to_string()));
                }
            }
            if let Some(otp) = OtpView::from_entry(entry) {
                // Tick supplies the timestamp; show a stable placeholder here.
                lines.push(Line::from(format!("otp: {} ({})", "------", &otp.uri)));
            }
            Text::from(lines)
        }
    };
    frame.render_widget(Paragraph::new(text).block(block), area);
}
