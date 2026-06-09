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
// Folder-prefix autocomplete helpers (Fix 3 — Create path field)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Create-form path prefix helper (Enhancement 1)
// ---------------------------------------------------------------------------

/// Compute the directory prefix to pre-fill the Path field when the user
/// opens the CREATE form from a given tree selection.
///
/// Rules:
/// - `selected` is `None` or the virtual root (`""`): returns `""`.
/// - `selected` is a real entry (leaf — `is_entry == true`):
///   returns everything up to and including the last `/`, e.g.
///   `"infra/mac-studio"` → `"infra/"`.  If the path has no `/`,
///   returns `""`.
/// - `selected` is a directory (full path, `is_entry == false`):
///   returns the path + `"/"`, e.g. `"infra"` → `"infra/"`.
///
/// The returned prefix is ready to use as the initial value of the Path input
/// so the user only has to type the entry name and press Enter.
pub fn create_path_prefix(selected: Option<&str>, is_entry: bool) -> String {
    match selected {
        None | Some("") => String::new(),
        Some(path) => {
            if is_entry {
                match path.rfind('/') {
                    Some(idx) => path[..=idx].to_string(),
                    None => String::new(),
                }
            } else {
                // Directory node: full-path id (e.g. "infra" or "a/b").
                format!("{path}/")
            }
        }
    }
}

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

    // ── create_path_prefix ───────────────────────────────────────────────────

    #[test]
    fn prefix_none_returns_empty() {
        assert_eq!(create_path_prefix(None, false), "");
        assert_eq!(create_path_prefix(None, true), "");
    }

    #[test]
    fn prefix_empty_string_returns_empty() {
        assert_eq!(create_path_prefix(Some(""), false), "");
        assert_eq!(create_path_prefix(Some(""), true), "");
    }

    #[test]
    fn prefix_leaf_with_dir_returns_dir_slash() {
        // e.g. user is on "infra/mac-studio" (a real entry)
        assert_eq!(create_path_prefix(Some("infra/mac-studio"), true), "infra/");
    }

    #[test]
    fn prefix_leaf_nested_returns_parent_slash() {
        assert_eq!(create_path_prefix(Some("a/b/entry"), true), "a/b/");
    }

    #[test]
    fn prefix_leaf_no_slash_returns_empty() {
        // Top-level entry (no directory segment)
        assert_eq!(create_path_prefix(Some("toplevel"), true), "");
    }

    #[test]
    fn prefix_directory_returns_path_slash() {
        // User is on the "infra" directory node
        assert_eq!(create_path_prefix(Some("infra"), false), "infra/");
    }

    #[test]
    fn prefix_nested_directory_returns_full_path_slash() {
        assert_eq!(create_path_prefix(Some("a/b"), false), "a/b/");
    }

    // ── OTP tests (existing) ──────────────────────────────────────────────────

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
