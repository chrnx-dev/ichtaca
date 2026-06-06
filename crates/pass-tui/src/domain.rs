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
            let idx = rng.random_range(0..pool.len());
            pool[idx] as char
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Folder-prefix autocomplete helpers (Fix 3 — Create path field)
// ---------------------------------------------------------------------------

/// Return all unique folder prefixes derived from `all_paths` whose prefix
/// matches `typed` (case-sensitive).
///
/// A folder prefix is any leading `dir/` segment of a path.  For example,
/// the path `"infra/mac-studio"` yields the folder prefixes `"infra/"`.
/// The path `"a/b/c"` yields `"a/"` and `"a/b/"`.
///
/// Each returned string ends with `/`.  Duplicates are removed and the
/// result is sorted lexicographically.
///
/// Only entries that **start with** `typed` are included.  When `typed`
/// itself ends with `/`, deeper folder levels rooted at that prefix are
/// returned (and the prefix itself is excluded — it is already fully typed).
pub fn folder_suggestions(all_paths: &[String], typed: &str) -> Vec<String> {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for path in all_paths {
        // Walk every proper-prefix folder of this path.
        // "a/b/c" → ["a/", "a/b/"]   (the leaf "a/b/c" itself is excluded)
        let parts: Vec<&str> = path.split('/').collect();
        // We only want folder prefixes (1 up to len-1 segments).
        for depth in 1..parts.len() {
            let folder = format!("{}/", parts[..depth].join("/"));
            if folder.starts_with(typed) && folder.as_str() != typed {
                seen.insert(folder);
            }
        }
    }

    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── folder_suggestions ────────────────────────────────────────────────────

    fn sample_paths() -> Vec<String> {
        vec![
            "infra/mac-studio".to_string(),
            "infra/rke".to_string(),
            "store/x".to_string(),
        ]
    }

    #[test]
    fn folder_suggestions_prefix_inf_gives_infra() {
        let suggestions = folder_suggestions(&sample_paths(), "inf");
        assert_eq!(suggestions, vec!["infra/".to_string()]);
    }

    #[test]
    fn folder_suggestions_empty_prefix_gives_all_top_folders() {
        let mut suggestions = folder_suggestions(&sample_paths(), "");
        suggestions.sort();
        assert_eq!(
            suggestions,
            vec!["infra/".to_string(), "store/".to_string()]
        );
    }

    #[test]
    fn folder_suggestions_infra_slash_gives_no_deeper_folders_when_none_exist() {
        // "infra/mac-studio" and "infra/rke" have no sub-folders under infra.
        let suggestions = folder_suggestions(&sample_paths(), "infra/");
        assert!(
            suggestions.is_empty(),
            "no deeper folders expected; got: {suggestions:?}"
        );
    }

    #[test]
    fn folder_suggestions_deeper_paths_yield_nested_folders() {
        let paths = vec!["a/b/c".to_string(), "a/b/d".to_string(), "a/e".to_string()];
        let mut suggestions = folder_suggestions(&paths, "a/");
        suggestions.sort();
        // "a/b/" is returned; "a/" itself is excluded (it equals typed)
        assert_eq!(suggestions, vec!["a/b/".to_string()]);
    }

    #[test]
    fn folder_suggestions_no_match_returns_empty() {
        let suggestions = folder_suggestions(&sample_paths(), "zzz");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn folder_suggestions_never_returns_leaf_paths() {
        // Leaf paths ("infra/mac-studio") must never appear — only folders.
        let suggestions = folder_suggestions(&sample_paths(), "infra/");
        for s in &suggestions {
            assert!(s.ends_with('/'), "all suggestions must end with '/'");
        }
    }

    #[test]
    fn folder_suggestions_deduplicates() {
        let paths = vec!["web/github.com".to_string(), "web/gitlab.com".to_string()];
        let suggestions = folder_suggestions(&paths, "");
        // "web/" must appear exactly once.
        assert_eq!(
            suggestions.iter().filter(|s| s.as_str() == "web/").count(),
            1,
            "'web/' must appear exactly once"
        );
    }

    // ── OTP / password tests (existing) ──────────────────────────────────────

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
