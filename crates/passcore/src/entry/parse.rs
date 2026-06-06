//! Loose-schema parsing with a byte-exact round-trip guarantee.
//!
//! Model: an entry is its exact sequence of lines plus a flag recording whether
//! the original ended in a newline. Parsing keeps every line verbatim; the
//! structured "view" (password, fields, otp, tags) is computed on top. Edits
//! mutate the underlying lines in place so unrecognized lines are never touched.

/// A parsed `pass` entry. Holds the original lines verbatim for round-tripping.
#[derive(Debug, Clone)]
pub struct Entry {
    lines: Vec<String>,
    trailing_newline: bool,
}

impl Entry {
    /// Parse raw decrypted text. Never fails: anything unrecognized is preserved.
    pub fn parse(input: &str) -> Self {
        let trailing_newline = input.ends_with('\n');
        let body = if trailing_newline {
            &input[..input.len() - 1]
        } else {
            input
        };
        // An empty body still represents zero lines; guard so "" -> [].
        let lines = if body.is_empty() && !trailing_newline {
            Vec::new()
        } else {
            body.split('\n').map(|s| s.to_string()).collect()
        };
        Self {
            lines,
            trailing_newline,
        }
    }

    /// Reconstruct the exact original text (byte-identical if unmodified).
    pub fn serialize(&self) -> String {
        let mut out = self.lines.join("\n");
        if self.trailing_newline {
            out.push('\n');
        }
        out
    }

    /// The password (first line), per the `pass` format.
    pub fn password(&self) -> &str {
        self.lines.first().map(String::as_str).unwrap_or("")
    }

    /// Replace the password (first line), leaving all other lines untouched.
    pub fn set_password(&mut self, value: &str) {
        if self.lines.is_empty() {
            self.lines.push(value.to_string());
        } else {
            self.lines[0] = value.to_string();
        }
    }

    /// Value of the first `key: value` line matching `key` (skips line 0).
    pub fn field(&self, key: &str) -> Option<&str> {
        self.lines.iter().skip(1).find_map(|line| {
            let (k, v) = line.split_once(':')?;
            (k.trim() == key).then_some(v.trim())
        })
    }

    /// Update an existing `key: value` line in place, or append one if absent.
    pub fn set_field(&mut self, key: &str, value: &str) {
        for line in self.lines.iter_mut().skip(1) {
            if let Some((k, _)) = line.split_once(':') {
                if k.trim() == key {
                    *line = format!("{key}: {value}");
                    return;
                }
            }
        }
        self.lines.push(format!("{key}: {value}"));
    }

    /// Remove the first `key: value` line (after the password line) whose key
    /// trims to `key`. Returns true if a line was removed. Other lines/order
    /// are preserved; line 0 (password) is never touched.
    pub fn remove_field(&mut self, key: &str) -> bool {
        let idx = self.lines.iter().enumerate().skip(1).find_map(|(i, line)| {
            let (k, _) = line.split_once(':')?;
            (k.trim() == key).then_some(i)
        });
        if let Some(i) = idx {
            self.lines.remove(i);
            true
        } else {
            false
        }
    }

    /// The first `otpauth://` line, if any.
    pub fn otp_uri(&self) -> Option<&str> {
        self.lines
            .iter()
            .map(String::as_str)
            .find(|l| l.trim_start().starts_with("otpauth://"))
            .map(str::trim)
    }

    /// Replace the first `otpauth://` line with `uri`, or append it when absent.
    /// Pass `None` to remove the first such line (no-op if none exists).
    pub fn set_otp(&mut self, uri: Option<&str>) {
        let pos = self
            .lines
            .iter()
            .position(|l| l.trim_start().starts_with("otpauth://"));
        match (uri, pos) {
            (Some(u), Some(i)) => self.lines[i] = u.to_string(),
            (Some(u), None) => self.lines.push(u.to_string()),
            (None, Some(i)) => {
                self.lines.remove(i);
            }
            (None, None) => {}
        }
    }

    /// Replace all dedicated `@tag` lines with a single new line built from
    /// `tags`.  A *dedicated tag line* is a non-empty line where every
    /// whitespace-separated token starts with `@`.  `key: value` field lines
    /// (even a `tags:` field) are never touched.
    ///
    /// Leading `@` on input tags is stripped so neither `"work"` nor `"@work"`
    /// produces `@@work`.  When `tags` is empty the lines are just removed.
    pub fn set_tags(&mut self, tags: &[String]) {
        // Remove all existing dedicated @-tag lines.
        self.lines.retain(|line| !is_dedicated_tag_line(line));
        // Append a new line if there is anything to write.
        if !tags.is_empty() {
            let line = tags
                .iter()
                .map(|t| format!("@{}", t.trim_start_matches('@')))
                .collect::<Vec<_>>()
                .join(" ");
            self.lines.push(line);
        }
    }

    /// Tags collected from an optional `tags:` field (comma-separated, optional
    /// leading `@`) and from `@token` occurrences on non-password lines.
    /// Deduplicated, first-seen order preserved.
    pub fn tags(&self) -> Vec<String> {
        use std::collections::HashSet;
        let mut tags = Vec::new();
        let mut seen = HashSet::new();
        if let Some(list) = self.field("tags") {
            for raw in list.split(',') {
                let t = raw.trim().trim_start_matches('@');
                if !t.is_empty() && seen.insert(t.to_string()) {
                    tags.push(t.to_string());
                }
            }
        }
        for line in self.lines.iter().skip(1) {
            // Skip the `tags:` field line itself; it was parsed above and its
            // `@`-prefixed words still carry comma punctuation.
            if matches!(line.split_once(':'), Some((k, _)) if k.trim() == "tags") {
                continue;
            }
            for tok in line.split_whitespace() {
                if let Some(tag) = tok.strip_prefix('@') {
                    if !tag.is_empty() && seen.insert(tag.to_string()) {
                        tags.push(tag.to_string());
                    }
                }
            }
        }
        tags
    }

    /// Trimmed keys of all `key: value` lines after line 0, excluding any
    /// `otpauth://` line and any dedicated `@tag` line (a non-empty line whose
    /// every whitespace token starts with `@`).
    pub fn field_keys(&self) -> Vec<String> {
        self.fields().into_iter().map(|(k, _)| k).collect()
    }

    /// Trimmed key+value pairs of all `key: value` lines after line 0, applying
    /// the same skipping rules as [`field_keys`](Self::field_keys): excludes the
    /// password line, any `otpauth://` line, and any dedicated `@tag` line.
    pub fn fields(&self) -> Vec<(String, String)> {
        self.lines
            .iter()
            .skip(1)
            .filter_map(|line| {
                if is_otp_line(line) || is_dedicated_tag_line(line) {
                    return None;
                }
                let (k, v) = line.split_once(':')?;
                let key = k.trim();
                (!key.is_empty()).then(|| (key.to_string(), v.trim().to_string()))
            })
            .collect()
    }
}

/// Returns `true` when `line` is an `otpauth://` URI line (ignoring leading
/// whitespace).
fn is_otp_line(line: &str) -> bool {
    line.trim_start().starts_with("otpauth://")
}

/// Returns `true` when every whitespace-separated token in `line` starts with
/// `@`.  An empty line is *not* a dedicated tag line.
fn is_dedicated_tag_line(line: &str) -> bool {
    let mut tokens = line.split_whitespace().peekable();
    // Must have at least one token and none of them may be a `key:` token.
    tokens.peek().is_some() && tokens.all(|t| t.starts_with('@'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(input: &str) {
        let e = Entry::parse(input);
        assert_eq!(e.serialize(), input, "round-trip must be byte-identical");
    }

    #[test]
    fn round_trip_trailing_newline() {
        rt("pw\nuser: bob\nurl: example.com\n");
    }

    #[test]
    fn round_trip_no_trailing_newline() {
        rt("pw\nuser: bob");
    }

    #[test]
    fn round_trip_blank_lines_and_unknown_lines() {
        rt("pw\n\nrandom note line\n@work @personal\notpauth://totp/x?secret=ABC\n");
    }

    #[test]
    fn round_trip_password_only() {
        rt("just-a-password\n");
    }

    #[test]
    fn password_is_first_line() {
        let e = Entry::parse("pw\nuser: bob\n");
        assert_eq!(e.password(), "pw");
    }

    #[test]
    fn fields_are_parsed_from_key_colon_value() {
        let e = Entry::parse("pw\nuser: bob\nurl: example.com\n");
        assert_eq!(e.field("user"), Some("bob"));
        assert_eq!(e.field("url"), Some("example.com"));
        assert_eq!(e.field("nope"), None);
    }

    #[test]
    fn otp_uri_is_detected() {
        let e = Entry::parse("pw\notpauth://totp/x?secret=ABC\n");
        assert_eq!(e.otp_uri(), Some("otpauth://totp/x?secret=ABC"));
    }

    #[test]
    fn tags_are_collected_from_at_tokens() {
        let e = Entry::parse("pw\n@work @personal\n");
        assert_eq!(e.tags(), vec!["work".to_string(), "personal".to_string()]);
    }

    #[test]
    fn set_field_updates_in_place_preserving_other_lines() {
        let mut e = Entry::parse("pw\nuser: bob\nrandom note\nurl: a.com\n");
        e.set_field("user", "alice");
        assert_eq!(e.serialize(), "pw\nuser: alice\nrandom note\nurl: a.com\n");
    }

    #[test]
    fn set_field_appends_when_key_absent() {
        let mut e = Entry::parse("pw\nuser: bob\n");
        e.set_field("url", "a.com");
        assert_eq!(e.serialize(), "pw\nuser: bob\nurl: a.com\n");
    }

    #[test]
    fn set_password_replaces_first_line_only() {
        let mut e = Entry::parse("oldpw\nuser: bob\n");
        e.set_password("newpw");
        assert_eq!(e.serialize(), "newpw\nuser: bob\n");
    }

    #[test]
    fn round_trip_empty_string() {
        rt("");
    }

    #[test]
    fn round_trip_single_newline() {
        rt("\n");
    }

    #[test]
    fn tags_from_tags_field_strip_at_and_dedup() {
        let e = Entry::parse("pw\ntags: @work, personal, work\n@work @other\n");
        assert_eq!(
            e.tags(),
            vec![
                "work".to_string(),
                "personal".to_string(),
                "other".to_string()
            ]
        );
    }

    #[test]
    fn tags_ignore_at_in_password_line() {
        let e = Entry::parse("@notatag\nuser: bob\n");
        assert!(e.tags().is_empty());
    }

    // ── set_otp ──────────────────────────────────────────────────────────────

    #[test]
    fn set_otp_replaces_existing() {
        let mut e = Entry::parse("pw\nuser: bob\notpauth://totp/x?secret=A\nnote\n");
        e.set_otp(Some("otpauth://totp/y?secret=B"));
        let s = e.serialize();
        assert!(s.contains("otpauth://totp/y?secret=B"), "new otp present");
        assert!(!s.contains("otpauth://totp/x?secret=A"), "old otp gone");
        assert!(s.contains("user: bob"), "user field intact");
        assert!(s.contains("note"), "note intact");
        // password first line unchanged
        assert_eq!(e.password(), "pw");
    }

    #[test]
    fn set_otp_appends_when_absent() {
        let mut e = Entry::parse("pw\nuser: bob\n");
        e.set_otp(Some("otpauth://totp/z?secret=C"));
        let s = e.serialize();
        assert!(s.contains("otpauth://totp/z?secret=C"), "otp appended");
        assert!(s.contains("user: bob"), "user field intact");
        assert_eq!(e.password(), "pw");
    }

    #[test]
    fn set_otp_none_removes() {
        let mut e = Entry::parse("pw\nuser: bob\notpauth://totp/x?secret=A\nnote\n");
        e.set_otp(None);
        let s = e.serialize();
        assert!(!s.contains("otpauth://"), "otp line gone");
        assert!(s.contains("user: bob"), "user field intact");
        assert!(s.contains("note"), "note intact");
        assert_eq!(e.password(), "pw");
        // calling None again is a no-op
        let before = e.serialize();
        e.set_otp(None);
        assert_eq!(e.serialize(), before, "second None is no-op");
    }

    // ── set_tags ─────────────────────────────────────────────────────────────

    #[test]
    fn set_tags_replaces_dedicated_line() {
        let mut e = Entry::parse("pw\n@old @x\nnote\n");
        e.set_tags(&["work".to_string(), "home".to_string()]);
        let s = e.serialize();
        assert!(s.contains("@work @home"), "@work @home line present");
        assert!(!s.contains("@old"), "@old gone");
        assert!(s.contains("note"), "note intact");
        assert_eq!(e.password(), "pw");
    }

    #[test]
    fn set_tags_strips_leading_at() {
        let mut e = Entry::parse("pw\n");
        e.set_tags(&["@work".to_string()]);
        let s = e.serialize();
        assert!(s.contains("@work"), "@work present");
        assert!(!s.contains("@@work"), "no double-at");
    }

    #[test]
    fn set_tags_empty_clears() {
        let mut e = Entry::parse("pw\n@a @b\nnote\n");
        e.set_tags(&[]);
        let s = e.serialize();
        assert!(!s.contains("@a"), "@a gone");
        assert!(!s.contains("@b"), "@b gone");
        assert!(s.contains("note"), "note intact");
        assert_eq!(e.password(), "pw");
    }

    #[test]
    fn set_tags_does_not_touch_tags_field() {
        let mut e = Entry::parse("pw\ntags: a, b\nnote\n");
        e.set_tags(&["x".to_string()]);
        let s = e.serialize();
        assert!(s.contains("tags: a, b"), "tags: field survives");
        assert!(s.contains("@x"), "@x appended");
        assert!(s.contains("note"), "note intact");
        assert_eq!(e.password(), "pw");
    }

    // ── remove_field ─────────────────────────────────────────────────────────

    #[test]
    fn remove_field_removes_matching_line() {
        let mut e = Entry::parse("pw\nuser: bob\nurl: x\nnote\n");
        assert!(e.remove_field("url"), "should return true when found");
        let s = e.serialize();
        assert!(!s.contains("url:"), "url line must be gone");
        assert!(s.contains("user: bob"), "user: bob must be intact");
        assert!(s.contains("note"), "note must be intact");
        assert_eq!(e.password(), "pw", "password must be intact");
    }

    #[test]
    fn remove_field_absent_returns_false() {
        let mut e = Entry::parse("pw\nuser: bob\nurl: x\n");
        let before = e.serialize();
        assert!(!e.remove_field("missing"), "absent key returns false");
        assert_eq!(e.serialize(), before, "entry must be unchanged");
    }

    #[test]
    fn remove_field_never_touches_password() {
        // Password line looks like a key: value pair; remove_field must ignore it.
        let mut e = Entry::parse("key: secret\nuser: bob\n");
        assert!(!e.remove_field("key"), "must not match password at index 0");
        assert_eq!(e.password(), "key: secret", "password line intact");
        assert!(e.serialize().contains("user: bob"), "user line intact");
    }

    #[test]
    fn remove_field_round_trip_unrelated_lines_intact() {
        let input = "pw\nuser: bob\nurl: example.com\nnote line\n";
        let mut e = Entry::parse(input);
        assert!(e.remove_field("url"));
        let s = e.serialize();
        // All unrelated lines are byte-identical in their original form.
        assert!(s.starts_with("pw\n"), "password line byte-identical");
        assert!(s.contains("user: bob"), "user field byte-identical");
        assert!(s.contains("note line"), "note line byte-identical");
        assert!(s.ends_with('\n'), "trailing newline preserved");
    }

    // ── field_keys / fields ────────────────────────────────────────────────────

    #[test]
    fn field_keys_returns_kv_keys_after_line0() {
        let e = Entry::parse("pw\nuser: bob\nurl: example.com\n");
        assert_eq!(e.field_keys(), vec!["user".to_string(), "url".to_string()]);
    }

    #[test]
    fn field_keys_skips_password_otp_and_tag_lines() {
        let e = Entry::parse(
            "pw\nuser: bob\notpauth://totp/x?secret=ABC\n@work @home\nurl: example.com\n",
        );
        assert_eq!(e.field_keys(), vec!["user".to_string(), "url".to_string()]);
    }

    #[test]
    fn field_keys_trims_keys() {
        let e = Entry::parse("pw\n  user : bob\n");
        assert_eq!(e.field_keys(), vec!["user".to_string()]);
    }

    #[test]
    fn fields_returns_trimmed_key_value_pairs() {
        let e = Entry::parse("pw\n  user :  bob \nurl: example.com\n");
        assert_eq!(
            e.fields(),
            vec![
                ("user".to_string(), "bob".to_string()),
                ("url".to_string(), "example.com".to_string()),
            ]
        );
    }

    #[test]
    fn fields_skips_password_otp_and_tag_lines() {
        let e = Entry::parse(
            "pw\nuser: bob\notpauth://totp/x?secret=ABC\n@work\nurl: example.com\n",
        );
        let f = e.fields();
        assert_eq!(f.len(), 2, "only user and url fields expected; got {f:?}");
        assert!(f.iter().any(|(k, v)| k == "user" && v == "bob"));
        assert!(f.iter().any(|(k, v)| k == "url" && v == "example.com"));
        assert!(
            !f.iter().any(|(_, v)| v.contains("otpauth")),
            "otp must not appear as a field; got {f:?}"
        );
    }
}
