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

    /// The first `otpauth://` line, if any.
    pub fn otp_uri(&self) -> Option<&str> {
        self.lines
            .iter()
            .map(String::as_str)
            .find(|l| l.trim_start().starts_with("otpauth://"))
            .map(str::trim)
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
}
