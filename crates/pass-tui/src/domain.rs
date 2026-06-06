//! Pure domain helpers reused across phases (no UI dependencies).

// ---------------------------------------------------------------------------
// OTP formatting — preserved from the old otp.rs
// ---------------------------------------------------------------------------

/// Group a 6-digit code as "123 456" for readability.
///
/// Any code whose length is not exactly 6 is returned unchanged.
#[allow(dead_code)] // used in Phase 2 (OTP detail panel)
pub fn format_code(code: &str) -> String {
    if code.len() == 6 {
        format!("{} {}", &code[..3], &code[3..])
    } else {
        code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_digit_code_is_grouped() {
        assert_eq!(format_code("123456"), "123 456");
        assert_eq!(format_code("282760"), "282 760");
    }

    #[test]
    fn non_six_digit_codes_pass_through() {
        assert_eq!(format_code("94287082"), "94287082");
        assert_eq!(format_code("12345"), "12345");
        assert_eq!(format_code(""), "");
    }
}
