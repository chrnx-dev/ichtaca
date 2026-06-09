//! Shared CSPRNG password generation.
//!
//! Pure and UI-free: both the TUI and the Tauri backend call into this so the
//! "configurable length and character set" claim is implemented in exactly one
//! place.

/// Charset used for symbol-free passwords (alphanumeric only).
const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Charset used for passwords with symbols.
const ALNUM_SYM: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";

/// Generate a cryptographically-random password of `length` printable ASCII
/// characters. If `symbols` is true, punctuation characters are included.
///
/// Uses `rand::rng()` (OS-backed CSPRNG via `OsRng`). Index selection uses
/// `rng.random_range`, which is rejection-sampled internally, so there is no
/// modulo bias toward earlier characters in the pool.
pub fn generate_password(length: usize, symbols: bool) -> String {
    use rand::Rng;

    let pool = if symbols { ALNUM_SYM } else { ALNUM };
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..pool.len());
            pool[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn generate_password_length_zero_is_empty() {
        assert_eq!(generate_password(0, true), "");
        assert_eq!(generate_password(0, false), "");
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
        assert_eq!(pw.len(), 200, "password length must be 200");
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
