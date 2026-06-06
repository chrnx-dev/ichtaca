//! Pure rendering: `render(frame, &AppState, now_unix)`. No state mutation, no I/O.

mod detail;
mod form;
mod modal;
mod search;
mod statusbar;
mod tree;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{AppState, Mode};
use crate::theme;

/// Draw the whole UI for the current state.
/// `now_unix` is the current unix timestamp in seconds; it is forwarded to the
/// detail panel so the OTP code and countdown are recomputed from the clock each
/// frame rather than from a cached value.
pub fn render(frame: &mut Frame, state: &AppState, now_unix: u64) {
    // Outer split: header (1 line) | body | statusbar (1 line)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // branding header
            Constraint::Min(1),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    // ── Branding header ──────────────────────────────────────────────────────
    let header = Paragraph::new(Line::from(vec![
        Span::styled("ICHTACA", theme::title()),
        Span::styled("  ·  lo oculto  ·", theme::hint()),
    ]))
    .style(Style::default().bg(theme::SURFACE).fg(theme::BG));
    frame.render_widget(header, outer[0]);

    // ── Main body ────────────────────────────────────────────────────────────
    let body = outer[1];

    if matches!(state.mode, Mode::Help) {
        modal::render_help(frame, body);
        statusbar::render(frame, outer[2], state);
        return;
    }

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(body);

    tree::render(frame, panels[0], state);
    detail::render(frame, panels[1], state, now_unix);
    statusbar::render(frame, outer[2], state);

    match &state.mode {
        Mode::Search => search::render(frame, body, state),
        Mode::EditForm => form::render(frame, body, state),
        Mode::Confirm(c) => modal::render_confirm(frame, body, c),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, NoticeKind};
    use passcore::{Entry, EntryNode};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Render the state into a 60×14 buffer and return a plain string for assertions.
    /// (14 rows: 1 header + 12 body + 1 status bar)
    /// `now_unix` is passed to the detail panel for deterministic OTP rendering in tests.
    fn draw_at(state: &AppState, now_unix: u64) -> String {
        let backend = TestBackend::new(60, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, state, now_unix)).unwrap();
        let buf = terminal.backend().buffer().clone();
        // Join all cell symbols row-by-row into a single string.
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    /// Convenience wrapper using timestamp 0 for tests that don't involve OTP.
    fn draw(state: &AppState) -> String {
        draw_at(state, 0)
    }

    fn browse_state() -> AppState {
        let mut s = AppState::new();
        s.roots = EntryNode::from_paths(&["web/github.com".to_string(), "email/work".to_string()]);
        s
    }

    /// Branding header shows "ICHTACA" in the first row.
    #[test]
    fn header_shows_ichtaca() {
        let s = browse_state();
        let out = draw(&s);
        let first_line = out.lines().next().unwrap_or("");
        assert!(
            first_line.contains("ICHTACA"),
            "expected 'ICHTACA' in header row, got: {first_line:?}"
        );
    }

    /// Branding header shows the subtitle in the first row.
    #[test]
    fn header_shows_subtitle() {
        let s = browse_state();
        let out = draw(&s);
        let first_line = out.lines().next().unwrap_or("");
        assert!(
            first_line.contains("lo oculto"),
            "expected 'lo oculto' in header row, got: {first_line:?}"
        );
    }

    /// Asserts the tree panel title, detail panel title, and top-level dirs are visible.
    #[test]
    fn browse_renders_tree_and_detail_titles() {
        let s = browse_state();
        let out = draw(&s);
        assert!(out.contains("Entries"), "missing 'Entries' title");
        assert!(out.contains("Detail"), "missing 'Detail' title");
        assert!(out.contains("email"), "missing 'email' dir");
        assert!(out.contains("web"), "missing 'web' dir");
    }

    /// Password is shown as bullet dots when hidden; the real value appears after reveal.
    #[test]
    fn detail_hides_password_until_revealed() {
        let mut s = browse_state();
        s.detail = Some(Entry::parse("hunter2\nuser: bob\n"));
        s.detail_path = Some("x".into());

        let hidden = draw(&s);
        assert!(
            !hidden.contains("hunter2"),
            "password should be hidden but 'hunter2' found"
        );
        assert!(
            hidden.contains('\u{2022}'),
            "hidden password should show bullet dots"
        );

        s.reveal = true;
        let shown = draw(&s);
        assert!(
            shown.contains("hunter2"),
            "revealed password 'hunter2' not found"
        );
    }

    /// An error notification replaces the hint bar with the message text.
    #[test]
    fn status_bar_shows_error_notification() {
        let mut s = browse_state();
        s.notify("boom", NoticeKind::Error);
        let out = draw(&s);
        assert!(
            out.contains("boom"),
            "notification 'boom' not found in status bar"
        );
    }

    /// The search overlay shows the "Search" title and the typed query characters.
    #[test]
    fn search_overlay_shows_query() {
        let mut s = browse_state();
        update_to_search(&mut s);
        let out = draw(&s);
        assert!(out.contains("Search"), "missing 'Search' overlay title");
        assert!(out.contains("gi"), "query 'gi' not found in search overlay");
    }

    /// Detail panel shows the live OTP code + countdown for an entry with a known URI.
    /// Secret JBSWY3DPEHPK3PXP at ts=0 → code 282760 → rendered "282 760", 30s remaining.
    #[test]
    fn detail_renders_live_otp_code_and_countdown() {
        const TEST_URI: &str = "otpauth://totp/x?secret=JBSWY3DPEHPK3PXP";
        let mut s = browse_state();
        s.detail = Some(passcore::Entry::parse(&format!("pw\n{TEST_URI}\n")));
        s.detail_path = Some("web/github.com".into());
        // ts=0: code=282760 → "282 760", 30s remaining
        let out = draw_at(&s, 0);
        assert!(
            out.contains("282 760"),
            "expected OTP code '282 760' in output, got:\n{out}"
        );
        assert!(
            out.contains("30s"),
            "expected '30s' countdown in output, got:\n{out}"
        );
    }

    fn update_to_search(s: &mut AppState) {
        crate::update::update(s, crate::action::Action::EnterSearch);
        crate::update::update(s, crate::action::Action::Input('g'));
        crate::update::update(s, crate::action::Action::Input('i'));
    }

    /// Confirm modal renders the "Confirm" title and the prompt text.
    #[test]
    fn confirm_modal_renders_prompt() {
        let mut s = browse_state();
        s.mode = crate::state::Mode::Confirm(crate::state::Confirm {
            prompt: "Delete web/github.com? (y/n)".into(),
            target: "web/github.com".into(),
            kind: crate::state::ConfirmKind::Delete,
        });
        let out = draw(&s);
        assert!(out.contains("Confirm"), "missing 'Confirm' modal title");
        assert!(
            out.contains("Delete web/github.com"),
            "confirm prompt text not found"
        );
    }
}
