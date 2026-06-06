//! Detail-panel OTP view: holds the entry's `otpauth://` URI and renders the
//! current code plus a countdown. Code computation is `passcore::otp`'s job.

use passcore::Entry;

/// A live OTP display derived from an entry's `otpauth://` URI.
#[derive(Debug, Clone)]
pub struct OtpView {
    pub uri: String,
}

impl OtpView {
    /// Build from an entry, if it carries an `otpauth://` URI.
    pub fn from_entry(entry: &Entry) -> Option<Self> {
        entry.otp_uri().map(|uri| Self {
            uri: uri.to_string(),
        })
    }

    /// The current code + seconds remaining, computed via the core for `now`.
    /// Returns `None` if the URI cannot be parsed.
    // Called when OTP timestamp is threaded through render (follow-up task).
    #[allow(dead_code)]
    pub fn current(&self, now_unix: u64) -> Option<(String, u64)> {
        // Real core API: `code_at` returns `Otp { code: String, seconds_remaining: u64 }`.
        // Use its fields directly — do NOT treat the `Otp` as a `&str`, and do NOT
        // re-derive the period (the core already accounts for the URI's period).
        let otp = passcore::otp::code_at(&self.uri, now_unix).ok()?;
        Some((format_code(&otp.code), otp.seconds_remaining))
    }
}

/// Group a 6-digit code as "123 456".
// Used by `current()` and the detail render; tested directly.
#[allow(dead_code)]
pub fn format_code(code: &str) -> String {
    if code.len() == 6 {
        format!("{} {}", &code[..3], &code[3..])
    } else {
        code.to_string()
    }
}

/// Seconds remaining in the current period (1..=period).
/// Kept for its own unit test; `current()` uses the value from `passcore::otp::code_at`.
#[allow(dead_code)]
pub fn seconds_remaining(now_unix: u64, period: u64) -> u64 {
    period - (now_unix % period)
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::Entry;

    // Known-good URI: JBSWY3DPEHPK3PXP = base32("HelloWorld\x00\x00"), 6 digits, 30 s period.
    // Deterministic vectors (computed from passcore::otp::code_at):
    //   ts=0:  code="282760", secs_remaining=30
    //   ts=59: code="996554", secs_remaining=1
    const TEST_URI: &str = "otpauth://totp/x?secret=JBSWY3DPEHPK3PXP";

    #[test]
    fn no_otp_uri_yields_no_view() {
        let e = Entry::parse("pw\nuser: bob\n");
        assert!(OtpView::from_entry(&e).is_none());
    }

    #[test]
    fn entry_with_otp_uri_yields_a_view() {
        let e = Entry::parse("pw\notpauth://totp/x?secret=JBSWY3DPEHPK3PXP\n");
        let v = OtpView::from_entry(&e);
        assert!(v.is_some());
    }

    #[test]
    fn formatted_code_is_six_digits_spaced() {
        // format_code groups 6 digits as "123 456" for readability.
        assert_eq!(format_code("123456"), "123 456");
    }

    #[test]
    fn seconds_remaining_is_in_period() {
        // For a 30s period, remaining is always 1..=30.
        for ts in [0u64, 15, 29, 30, 31, 59, 60] {
            let r = seconds_remaining(ts, 30);
            assert!((1..=30).contains(&r), "ts={ts} r={r}");
        }
    }

    #[test]
    fn current_returns_deterministic_code_at_known_timestamp() {
        let e = Entry::parse(&format!("pw\n{TEST_URI}\n"));
        let view = OtpView::from_entry(&e).expect("URI should be found");

        // ts=0: first window, code=282760, 30 s remaining.
        let (code, secs) = view.current(0).expect("code_at should succeed");
        assert_eq!(code, "282 760", "formatted code at ts=0");
        assert_eq!(secs, 30, "seconds_remaining at ts=0");

        // ts=59: second window (counter=1), code=996554, 1 s remaining.
        let (code59, secs59) = view.current(59).expect("code_at should succeed");
        assert_eq!(code59, "996 554", "formatted code at ts=59");
        assert_eq!(secs59, 1, "seconds_remaining at ts=59");
    }

    #[test]
    fn non_six_digit_code_passes_through_unchanged() {
        // format_code must not mangle codes that aren't exactly 6 chars.
        assert_eq!(format_code("94287082"), "94287082");
        assert_eq!(format_code("12345"), "12345");
    }
}
