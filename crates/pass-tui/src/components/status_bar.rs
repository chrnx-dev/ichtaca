//! StatusBar component — hint / status line shown at the bottom of the screen.
//!
//! In Phase 1 it shows the static browse-mode hint line.
//! It holds the global `q` / Ctrl-C subscription target so the quit event is
//! routed here when the user is not explicitly focused on any component.

use tui_realm_stdlib::components::Paragraph;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, HorizontalAlignment, QueryResult, Style, TextModifiers,
};
use tuirealm::ratatui::text::{Line, Span};
use tuirealm::state::State;

use crate::msg::Msg;
use crate::theme;
use crate::theme::icons;

/// Hint/status bar shown at the bottom of the window.
pub struct StatusBar {
    inner: Paragraph,
}

impl Default for StatusBar {
    fn default() -> Self {
        // Browse-mode hint line (Phase 2).
        // Format: key(gold bold) label(muted)  key label  ...
        let hint_line = Line::from(vec![
            Span::styled(
                " ↑↓",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled("/jk", Style::default().fg(theme::MUTED_BRIGHT)),
            Span::styled(" move", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "←→",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled("/hl", Style::default().fg(theme::MUTED_BRIGHT)),
            Span::styled(" fold", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "c",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" copy", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "s",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" reveal", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                format!("/ {}", icons::SEARCH),
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" search", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "a",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" add", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "e",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" edit", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "d",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" del", Style::default().fg(theme::MUTED)),
            Span::raw("  "),
            Span::styled(
                "q",
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(TextModifiers::BOLD),
            ),
            Span::styled(" quit", Style::default().fg(theme::MUTED)),
        ]);

        let inner = Paragraph::default()
            .background(theme::SURFACE)
            .foreground(theme::MUTED)
            .alignment_horizontal(HorizontalAlignment::Left)
            .text(vec![hint_line]);

        Self { inner }
    }
}

impl Component for StatusBar {
    fn view(
        &mut self,
        frame: &mut tuirealm::ratatui::Frame,
        area: tuirealm::ratatui::layout::Rect,
    ) {
        self.inner.view(frame, area);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        self.inner.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.inner.attr(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.inner.perform(cmd)
    }
}

impl AppComponent<Msg, NoUserEvent> for StatusBar {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::GitSync),
            _ => Some(Msg::None),
        }
    }
}

// ── Git chip ────────────────────────────────────────────────────────────────

/// Spans for the right-aligned git chip: ` main ↑2 ↓1 ●3`.
///
/// `↓behind` is only as fresh as the last fetch — see `passcore::git::Status`.
/// Rendered by `Model::render_frame`, not mounted as a component: it is pure
/// display with no events of its own.
///
/// ponytail: no chip at all when the store is not a git repo — the caller
/// passes `None` and the footer keeps its full width.
pub fn git_chip(status: &passcore::git::Status) -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled(
        format!(" {} {}", icons::BRANCH, status.branch),
        Style::default().fg(theme::MUTED_BRIGHT),
    )];
    if status.ahead > 0 {
        spans.push(Span::styled(
            format!(" ↑{}", status.ahead),
            Style::default()
                .fg(theme::GOLD)
                .add_modifier(TextModifiers::BOLD),
        ));
    }
    if status.behind > 0 {
        spans.push(Span::styled(
            format!(" ↓{}", status.behind),
            Style::default().fg(theme::TURQUOISE),
        ));
    }
    if status.dirty > 0 {
        spans.push(Span::styled(
            format!(" ●{}", status.dirty),
            Style::default().fg(theme::COCHINEAL),
        ));
    }
    if !status.upstream {
        spans.push(Span::styled(
            " no remote",
            Style::default().fg(theme::MUTED),
        ));
    } else if status.ahead == 0 && status.behind == 0 && status.dirty == 0 {
        spans.push(Span::styled(" ✓", Style::default().fg(theme::JADE)));
    } else {
        // Only advertise the sync key when there is actually something to sync.
        spans.push(Span::styled(
            " ^g",
            Style::default()
                .fg(theme::GOLD)
                .add_modifier(TextModifiers::BOLD),
        ));
    }
    spans.push(Span::raw(" "));
    spans
}

/// Cell width of [`git_chip`], for the footer layout split.
pub fn git_chip_width(status: &passcore::git::Status) -> u16 {
    git_chip(status)
        .iter()
        .map(|s| s.content.chars().count())
        .sum::<usize>() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(status: &passcore::git::Status) -> String {
        git_chip(status)
            .iter()
            .map(|s| s.content.to_string())
            .collect()
    }

    #[test]
    fn clean_synced_repo_shows_branch_and_check() {
        let st = passcore::git::Status {
            branch: "main".into(),
            upstream: true,
            ..Default::default()
        };
        let t = text(&st);
        assert!(t.contains("main"), "branch name must show: {t}");
        assert!(t.contains('✓'), "clean+synced must show a check: {t}");
        assert!(!t.contains('↑'), "nothing to push: {t}");
    }

    #[test]
    fn unpushed_commits_show_ahead_count() {
        let st = passcore::git::Status {
            branch: "main".into(),
            ahead: 3,
            upstream: true,
            ..Default::default()
        };
        let t = text(&st);
        assert!(t.contains("↑3"), "ahead count must show: {t}");
        assert!(!t.contains('✓'), "not in sync, no check: {t}");
        assert!(
            t.contains("^g"),
            "sync key is advertised when actionable: {t}"
        );
    }

    #[test]
    fn missing_upstream_is_called_out() {
        let st = passcore::git::Status {
            branch: "main".into(),
            upstream: false,
            ..Default::default()
        };
        assert!(text(&st).contains("no remote"));
    }

    #[test]
    fn width_matches_rendered_text() {
        let st = passcore::git::Status {
            branch: "main".into(),
            ahead: 2,
            upstream: true,
            ..Default::default()
        };
        assert_eq!(git_chip_width(&st) as usize, text(&st).chars().count());
    }
}
