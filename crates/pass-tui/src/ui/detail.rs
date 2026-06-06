//! Right panel: the selected entry's detail. Password hidden unless revealed.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::otp::OtpView;
use crate::state::AppState;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, now_unix: u64) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("Detail", theme::title()));

    let text = match &state.detail {
        None => ratatui::text::Text::from(Line::from(vec![Span::styled(
            "Select an entry",
            theme::hint(),
        )])),
        Some(entry) => {
            let mut lines: Vec<Line> = Vec::new();

            // Password row ─────────────────────────────────────────────────
            let (pw_value_span, label_style) = if state.reveal {
                (
                    Span::styled(entry.password().to_string(), theme::revealed()),
                    theme::key(),
                )
            } else {
                (
                    Span::styled(
                        "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                        theme::key(),
                    ),
                    theme::key(),
                )
            };
            lines.push(Line::from(vec![
                Span::styled("password", label_style),
                Span::styled(": ", theme::key()),
                pw_value_span,
            ]));

            // Known fields (skip line 0 = password) ─────────────────────────
            for raw in entry.serialize().lines().skip(1) {
                if raw.trim_start().starts_with("otpauth://") {
                    continue; // shown separately below
                }
                if !raw.is_empty() {
                    // Split "key: value" into styled spans where possible.
                    if let Some((k, v)) = raw.split_once(": ") {
                        // "tags:" entries receive jade colour for their values.
                        let value_style = if k.trim() == "tags" {
                            theme::tag()
                        } else {
                            theme::text()
                        };
                        lines.push(Line::from(vec![
                            Span::styled(k.to_string(), theme::key()),
                            Span::styled(": ", theme::key()),
                            Span::styled(v.to_string(), value_style),
                        ]));
                    } else {
                        lines.push(Line::from(Span::styled(raw.to_string(), theme::text())));
                    }
                }
            }

            // OTP row ────────────────────────────────────────────────────────
            if let Some(otp) = OtpView::from_entry(entry) {
                let otp_line = match otp.current(now_unix) {
                    Some((code, secs)) => Line::from(vec![
                        Span::styled("otp", theme::key()),
                        Span::styled(": ", theme::key()),
                        Span::styled(code, theme::otp()),
                        Span::styled(" (", theme::key()),
                        Span::styled(format!("{secs}s"), theme::otp_countdown()),
                        Span::styled(")", theme::key()),
                    ]),
                    None => Line::from(vec![
                        Span::styled("otp", theme::key()),
                        Span::styled(": [invalid uri]", theme::error()),
                    ]),
                };
                lines.push(otp_line);
            }

            ratatui::text::Text::from(lines)
        }
    };
    frame.render_widget(Paragraph::new(text).block(block), area);
}
