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

// ---------------------------------------------------------------------------
// CSPRNG password generation (Phase 3)
// ---------------------------------------------------------------------------

/// Charset used for symbol-free passwords (alphanumeric only).
const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Charset used for passwords with symbols.
const ALNUM_SYM: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";

/// Generate a cryptographically-random password of `len` printable ASCII
/// characters. If `symbols` is true, punctuation characters are included.
///
/// Uses `rand::rng()` (OS-backed CSPRNG via OsRng).
pub fn generate_password(len: usize, symbols: bool) -> String {
    use rand::Rng;

    let pool = if symbols { ALNUM_SYM } else { ALNUM };
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx: usize = rng.random::<u64>() as usize % pool.len();
            pool[idx] as char
        })
        .collect()
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

    #[test]
    fn generate_password_correct_length() {
        for len in [0, 1, 8, 16, 32, 64] {
            let pw = generate_password(len, false);
            assert_eq!(
                pw.len(),
                len,
                "alnum password length mismatch for len={len}"
            );
        }
    }

    #[test]
    fn generate_password_alnum_only() {
        let pw = generate_password(200, false);
        assert!(
            pw.chars().all(|c| c.is_ascii_alphanumeric()),
            "no-symbols password must be alphanumeric only"
        );
    }

    #[test]
    fn generate_password_with_symbols_uses_extended_charset() {
        // Statistically, a 200-char password from ALNUM_SYM will almost certainly
        // contain at least one symbol character.
        let pw = generate_password(200, true);
        assert!(
            pw.len() == 200,
            "password length must be 200; got {}",
            pw.len()
        );
        assert!(
            pw.chars().all(|c| c.is_ascii_graphic()),
            "all chars must be printable ASCII"
        );
    }

    #[test]
    fn generate_password_uses_csprng_not_constant() {
        // Two independently generated passwords of 16 chars must differ
        // (with overwhelming probability — the chance of collision is negligible).
        let a = generate_password(16, true);
        let b = generate_password(16, true);
        assert_ne!(a, b, "two generated passwords must not be identical");
    }
}
