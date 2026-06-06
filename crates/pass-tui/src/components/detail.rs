//! Detail panel component — shows the fields of the selected entry.
//!
//! This is a **display-only** component that wraps a `Paragraph`.  The panel
//! content is pushed from the `Model` by calling `Detail::set_entry`.  No
//! key-handling is done here — global keys (c, s, q) are handled either by the
//! focused `EntryTree` component or by the StatusBar global subscriptions.
//!
//! Rendered layout (one line per item):
//!
//! ```text
//! path/to/entry
//!
//! password  ••••••••           (muted)  [or revealed value in gold_bright]
//! user      bob                (text)
//! url       example.com        (text)
//!
//! OTP       282 760  29s       (turquoise_bright + turquoise for countdown)
//!
//! tags      work  personal     (jade)
//! ```

use tui_realm_stdlib::components::Paragraph;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, LineStatic, QueryResult,
    SpanStatic, Style, TextModifiers,
};
use tuirealm::state::State;

use crate::domain;
use crate::msg::Msg;
use crate::theme;

/// Right-panel detail component.
pub struct Detail {
    inner: Paragraph,
}

impl Default for Detail {
    fn default() -> Self {
        let inner = Paragraph::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::MUTED)
                    .modifiers(BorderType::Rounded),
            )
            .alignment_horizontal(HorizontalAlignment::Left)
            .text(vec![empty_hint_line()]);
        Self { inner }
    }
}

impl Detail {
    /// Render the entry using current detail state.
    ///
    /// `entry`   — the parsed `Entry` (password, fields, tags, otp_uri).
    /// `reveal`  — whether the password is shown in plain text.
    /// `otp`     — pre-computed OTP (pass `None` when no otp_uri / error).
    /// `notice`  — an optional one-line status/info notice (e.g. "copied").
    #[allow(dead_code)] // used via build_lines_pub / model
    pub fn set_entry(
        &mut self,
        path: &str,
        entry: &passcore::Entry,
        reveal: bool,
        otp: Option<&passcore::Otp>,
        notice: Option<&str>,
    ) {
        let lines = build_lines(path, entry, reveal, otp, notice);
        self.inner.attr(
            Attribute::Text,
            AttrValue::Text(tuirealm::props::TextStatic::from(
                lines.into_iter().collect::<Vec<LineStatic>>(),
            )),
        );
    }

    /// Clear the detail panel (show placeholder text).
    #[allow(dead_code)] // used via push_detail_clear in model
    pub fn clear(&mut self) {
        self.inner.attr(
            Attribute::Text,
            AttrValue::Text(tuirealm::props::TextStatic::from(vec![empty_hint_line()])),
        );
    }
}

// ── Component forwarding ──────────────────────────────────────────────────────

impl Component for Detail {
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

impl AppComponent<Msg, NoUserEvent> for Detail {
    fn on(&mut self, _ev: &Event<NoUserEvent>) -> Option<Msg> {
        // Detail is display-only; all keys are handled by the Tree or global subs.
        Some(Msg::None)
    }
}

// ── Line builders ─────────────────────────────────────────────────────────────

/// Public re-export for use by `Model::refresh_detail`.
pub fn empty_hint_line_pub() -> LineStatic {
    empty_hint_line()
}

/// Public re-export for use by `Model::refresh_detail`.
pub fn build_lines_pub(
    path: &str,
    entry: &passcore::Entry,
    reveal: bool,
    otp: Option<&passcore::Otp>,
    notice: Option<&str>,
) -> Vec<LineStatic> {
    build_lines(path, entry, reveal, otp, notice)
}

fn empty_hint_line() -> LineStatic {
    LineStatic::from(vec![SpanStatic::styled(
        "  Select an entry to view its details",
        Style::default().fg(theme::MUTED),
    )])
}

fn kv_line(key: String, value_spans: Vec<SpanStatic>) -> LineStatic {
    let mut parts: Vec<SpanStatic> = vec![
        SpanStatic::styled(format!("  {key:<12}"), Style::default().fg(theme::MUTED)),
        SpanStatic::raw("  "),
    ];
    parts.extend(value_spans);
    LineStatic::from(parts)
}

fn build_lines(
    path: &str,
    entry: &passcore::Entry,
    reveal: bool,
    otp: Option<&passcore::Otp>,
    notice: Option<&str>,
) -> Vec<LineStatic> {
    let mut lines: Vec<LineStatic> = Vec::new();

    // ── Path header ──────────────────────────────────────────────────────
    lines.push(LineStatic::from(vec![SpanStatic::styled(
        format!("  {path}"),
        Style::default()
            .fg(theme::GOLD)
            .add_modifier(TextModifiers::BOLD),
    )]));
    lines.push(LineStatic::from(vec![SpanStatic::raw("")]));

    // ── Password ─────────────────────────────────────────────────────────
    let pw_spans = if reveal {
        vec![SpanStatic::styled(
            entry.password().to_string(),
            Style::default().fg(theme::GOLD_BRIGHT),
        )]
    } else {
        vec![SpanStatic::styled(
            "••••••••",
            Style::default().fg(theme::MUTED),
        )]
    };
    lines.push(kv_line("password".to_string(), pw_spans));

    // ── Fields ────────────────────────────────────────────────────────────
    for (key, val) in entry.fields() {
        lines.push(kv_line(
            key,
            vec![SpanStatic::styled(val, Style::default().fg(theme::TEXT))],
        ));
    }

    // ── OTP ───────────────────────────────────────────────────────────────
    if let Some(o) = otp {
        lines.push(LineStatic::from(vec![SpanStatic::raw("")]));
        let code_str = domain::format_code(&o.code);
        let secs_str = format!(" {}s", o.seconds_remaining);
        lines.push(LineStatic::from(vec![
            SpanStatic::styled(
                format!("  {:<14}", "otp"),
                Style::default().fg(theme::MUTED),
            ),
            SpanStatic::raw("  "),
            SpanStatic::styled(code_str, Style::default().fg(theme::TURQUOISE_BRIGHT)),
            SpanStatic::styled(secs_str, Style::default().fg(theme::TURQUOISE)),
        ]));
    }

    // ── Tags ──────────────────────────────────────────────────────────────
    let tags = entry.tags();
    if !tags.is_empty() {
        lines.push(LineStatic::from(vec![SpanStatic::raw("")]));
        let mut tag_spans: Vec<SpanStatic> = vec![
            SpanStatic::styled(
                format!("  {:<14}", "tags"),
                Style::default().fg(theme::MUTED),
            ),
            SpanStatic::raw("  "),
        ];
        for (i, tag) in tags.iter().enumerate() {
            if i > 0 {
                tag_spans.push(SpanStatic::raw("  "));
            }
            tag_spans.push(SpanStatic::styled(
                tag.clone(),
                Style::default().fg(theme::JADE),
            ));
        }
        lines.push(LineStatic::from(tag_spans));
    }

    // ── Notice ────────────────────────────────────────────────────────────
    if let Some(n) = notice {
        lines.push(LineStatic::from(vec![SpanStatic::raw("")]));
        lines.push(LineStatic::from(vec![SpanStatic::styled(
            format!("  {n}"),
            Style::default().fg(theme::JADE),
        )]));
    }

    lines
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::Entry;

    fn flat_text(lines: &[LineStatic]) -> String {
        lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }

    fn entry_with_otp() -> Entry {
        Entry::parse("s3cr3t\nuser: alice\notpauth://totp/x?secret=GEZDGNBVGY3TQOJQ\n@work\n")
    }

    #[test]
    fn password_masked_by_default() {
        let e = Entry::parse("hunter2\nuser: bob\n");
        let lines = build_lines("web/site", &e, false, None, None);
        let flat = flat_text(&lines);
        assert!(flat.contains("••••••••"), "password must be masked");
        assert!(!flat.contains("hunter2"), "plaintext must not appear");
    }

    #[test]
    fn password_revealed_when_flag_true() {
        let e = Entry::parse("hunter2\nuser: bob\n");
        let lines = build_lines("web/site", &e, true, None, None);
        let flat = flat_text(&lines);
        assert!(
            flat.contains("hunter2"),
            "plaintext must appear when revealed"
        );
        assert!(
            !flat.contains("••••••••"),
            "mask must not appear when revealed"
        );
    }

    #[test]
    fn otp_displayed_when_provided() {
        let e = entry_with_otp();
        let otp = passcore::Otp {
            code: "123456".to_string(),
            seconds_remaining: 17,
        };
        let lines = build_lines("web/site", &e, false, Some(&otp), None);
        let flat = flat_text(&lines);
        assert!(flat.contains("123 456"), "formatted OTP code must appear");
        assert!(flat.contains("17s"), "countdown must appear");
    }

    #[test]
    fn otp_absent_when_none() {
        let e = Entry::parse("pw\nuser: alice\n");
        let lines = build_lines("web/site", &e, false, None, None);
        let flat = flat_text(&lines);
        assert!(!flat.contains("123"), "OTP section must not appear");
    }

    #[test]
    fn tags_displayed() {
        let e = Entry::parse("pw\n@work @personal\n");
        let lines = build_lines("web/site", &e, false, None, None);
        let flat = flat_text(&lines);
        assert!(flat.contains("work"), "tags must appear");
        assert!(flat.contains("personal"), "all tags must appear");
    }

    #[test]
    fn notice_line_displayed() {
        let e = Entry::parse("pw\n");
        let lines = build_lines("web/site", &e, false, None, Some("copied (clears in 45s)"));
        let flat = flat_text(&lines);
        assert!(
            flat.contains("copied (clears in 45s)"),
            "notice must appear"
        );
    }

    #[test]
    fn fields_displayed() {
        let e = Entry::parse("pw\nuser: bob\nurl: example.com\n");
        let lines = build_lines("web/site", &e, false, None, None);
        let flat = flat_text(&lines);
        assert!(flat.contains("bob"), "field value must appear");
        assert!(flat.contains("example.com"), "second field must appear");
    }

    #[test]
    fn path_shown_as_header() {
        let e = Entry::parse("pw\n");
        let lines = build_lines("web/github.com", &e, false, None, None);
        let flat = flat_text(&lines);
        assert!(flat.contains("web/github.com"), "entry path must be shown");
    }
}
