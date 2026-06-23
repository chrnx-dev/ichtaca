//! CLI subcommand implementations. Each reuses `passcore`; no store/parse logic
//! is duplicated here.

use super::{CliError, Command};

type CliResult = Result<(), CliError>;

/// Run a non-`Doctor` subcommand.
pub fn run(cmd: Command) -> CliResult {
    let config = passcore::Config::load().unwrap_or_default();
    match cmd {
        Command::List => list(&config),
        Command::Search { query } => search(&config, &query),
        Command::Get { path } => get(&config, &path),
        Command::Show { path, json } => show(&config, &path, json),
        Command::Otp { path } => otp(&config, &path),
        Command::Copy { path } => copy(&config, &path),
        Command::Generate {
            path,
            length,
            no_symbols,
        } => generate(&config, &path, length, no_symbols),
        Command::Set {
            path,
            password_stdin,
            fields,
            tags,
            remove_fields,
            remove_tags,
        } => set(
            &config,
            &path,
            password_stdin,
            &fields,
            &tags,
            &remove_fields,
            &remove_tags,
        ),
        Command::Doctor => unreachable!("Doctor is handled in dispatch"),
    }
}

fn list(config: &passcore::Config) -> CliResult {
    let init = passcore::init_store(config)?;
    let paths = init.store.list()?;
    for path in paths {
        println!("{path}");
    }
    Ok(())
}

fn search(config: &passcore::Config, query: &str) -> CliResult {
    let init = passcore::init_store(config)?;
    let paths = init.store.list()?;
    let hits = passcore::fuzzy_paths(query, &paths);
    for hit in hits {
        println!("{}", hit.path);
    }
    Ok(())
}

fn get(config: &passcore::Config, path: &str) -> CliResult {
    let init = passcore::init_store(config)?;
    let entry = init.store.show(path)?;
    print!("{}", entry.password());
    Ok(())
}

fn otp(config: &passcore::Config, path: &str) -> CliResult {
    let init = passcore::init_store(config)?;
    let entry = init.store.show(path)?;
    match entry.otp_uri() {
        None => Err(CliError::Usage(format!("no OTP configured for {path}"))),
        Some(uri) => {
            let otp = passcore::current(uri)?;
            println!("{}", otp.code);
            Ok(())
        }
    }
}

fn copy(config: &passcore::Config, path: &str) -> CliResult {
    let init = passcore::init_store(config)?;
    let entry = init.store.show(path)?;
    let secret = passcore::Secret::from(entry.password());
    let backend = passcore::clipboard::default_backend()?;
    passcore::clipboard::copy_with(backend.as_ref(), &secret)?;
    let secs = config.clipboard.clear_after;
    if secs == 0 {
        println!("Copied {path} to clipboard.");
    } else {
        println!("Copied {path} to clipboard. Clearing in {secs}s (Ctrl-C to keep it).");
        std::thread::sleep(std::time::Duration::from_secs(secs));
        passcore::clipboard::clear_if_owned(backend.as_ref(), secret.expose_str())?;
    }
    Ok(())
}

fn generate(config: &passcore::Config, path: &str, length: usize, no_symbols: bool) -> CliResult {
    let mut init = passcore::init_store(config)?;
    if init.store.list()?.iter().any(|p| p == path) {
        return Err(CliError::Usage(format!(
            "entry already exists: {path} (refusing to overwrite)"
        )));
    }
    let secret = init.store.generate(path, length, !no_symbols)?;
    print!("{}", secret.expose_str());
    Ok(())
}

fn set(
    config: &passcore::Config,
    path: &str,
    password_stdin: bool,
    fields: &[String],
    tags: &[String],
    remove_fields: &[String],
    remove_tags: &[String],
) -> CliResult {
    if !password_stdin
        && fields.is_empty()
        && tags.is_empty()
        && remove_fields.is_empty()
        && remove_tags.is_empty()
    {
        return Err(CliError::Usage(
            "nothing to set: pass --password-stdin, --field key=value, --tag, \
             --remove-field, and/or --remove-tag"
                .into(),
        ));
    }
    let parsed: Vec<(String, String)> = fields
        .iter()
        .map(|f| parse_field(f))
        .collect::<Result<_, _>>()?;

    let mut init = passcore::init_store(config)?;
    // Read-modify-write: start from the existing entry if present, else a blank one.
    // This preserves any existing OTP, tags, and other fields.
    let mut entry = match init.store.show(path) {
        Ok(e) => e,
        Err(passcore::PassError::EntryNotFound(_)) => passcore::Entry::parse(""),
        Err(e) => return Err(e.into()),
    };
    let pw = if password_stdin {
        Some(read_stdin_trimmed()?)
    } else {
        None
    };
    apply_set(
        &mut entry,
        pw.as_deref(),
        &parsed,
        remove_fields,
        tags,
        remove_tags,
    );
    init.store
        .insert(path, &passcore::Secret::from(entry.serialize()), true)?;
    Ok(())
}

/// Apply password, field, and tag mutations to an entry. Extracted for unit testing.
///
/// Order of operations:
/// 1. Set password (if provided)
/// 2. Set/add fields
/// 3. Remove fields (nonexistent key → silent no-op)
/// 4. Merge tags: existing + add_tags (dedup, preserve order) − remove_tags;
///    only calls `set_tags` when add_tags or remove_tags is non-empty.
fn apply_set(
    entry: &mut passcore::Entry,
    password: Option<&str>,
    fields: &[(String, String)],
    remove_fields: &[String],
    add_tags: &[String],
    remove_tags: &[String],
) {
    if let Some(pw) = password {
        entry.set_password(pw);
    }
    for (k, v) in fields {
        entry.set_field(k, v);
    }
    for k in remove_fields {
        entry.remove_field(k);
    }
    if !add_tags.is_empty() || !remove_tags.is_empty() {
        let mut final_tags: Vec<String> = entry.tags();
        for tag in add_tags {
            let t = tag.trim_start_matches('@').trim().to_string();
            if !t.is_empty() && !final_tags.iter().any(|x| x == &t) {
                final_tags.push(t);
            }
        }
        final_tags.retain(|t| {
            !remove_tags
                .iter()
                .any(|r| r.trim_start_matches('@').trim() == t)
        });
        entry.set_tags(&final_tags);
    }
}

/// Parse a `key=value` field argument. The split occurs on the FIRST `=` only,
/// so `k=v=w` yields `("k", "v=w")`. Empty key or missing `=` is an error.
fn parse_field(s: &str) -> Result<(String, String), CliError> {
    match s.split_once('=') {
        Some(("", _)) => Err(CliError::Usage(format!(
            "invalid --field {s:?}, expected key=value"
        ))),
        Some((k, v)) => Ok((k.to_string(), v.to_string())),
        None => Err(CliError::Usage(format!(
            "invalid --field {s:?}, expected key=value"
        ))),
    }
}

/// Strip exactly one trailing `\n` (and a preceding `\r` if present).
/// `"pw\n"` → `"pw"`, `"pw\r\n"` → `"pw"`, `"pw"` → `"pw"`, `"a\nb\n"` → `"a\nb"`.
fn strip_one_trailing_newline(s: String) -> String {
    if let Some(stripped) = s.strip_suffix('\n') {
        stripped.strip_suffix('\r').unwrap_or(stripped).to_string()
    } else {
        s
    }
}

/// Read all of stdin, strip exactly one trailing newline.
fn read_stdin_trimmed() -> Result<String, CliError> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| CliError::Pass(passcore::PassError::Io(e)))?;
    Ok(strip_one_trailing_newline(buf))
}

fn show(config: &passcore::Config, path: &str, json: bool) -> CliResult {
    let init = passcore::init_store(config)?;
    let entry = init.store.show(path)?;
    if json {
        println!("{}", show_json(&entry, path));
    } else {
        print!("{}", show_plain(&entry, path));
    }
    Ok(())
}

/// Format an entry as a JSON metadata object. SECURITY: omits password and
/// raw otpauth URI — only `path`, `fields`, `tags`, and `has_otp` are emitted.
pub(crate) fn show_json(entry: &passcore::Entry, path: &str) -> String {
    let fields: Vec<serde_json::Value> = entry
        .fields()
        .into_iter()
        .map(|(k, v)| serde_json::json!([k, v]))
        .collect();
    let tags: Vec<String> = entry.tags();
    let has_otp = entry.otp_uri().is_some();
    serde_json::json!({
        "path": path,
        "fields": fields,
        "tags": tags,
        "has_otp": has_otp,
    })
    .to_string()
}

/// Format an entry as human-readable metadata text. SECURITY: omits password
/// and raw otpauth URI — only path, fields, tags, and otp presence are emitted.
pub(crate) fn show_plain(entry: &passcore::Entry, path: &str) -> String {
    let mut out = String::new();
    out.push_str(path);
    out.push('\n');
    for (key, value) in entry.fields() {
        out.push_str(&format!("{key}: {value}\n"));
    }
    let tags = entry.tags();
    if !tags.is_empty() {
        out.push_str(&format!("tags: {}\n", tags.join(", ")));
    }
    let otp_label = if entry.otp_uri().is_some() {
        "yes"
    } else {
        "no"
    };
    out.push_str(&format!("otp: {otp_label}\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::Entry;

    // ── parse_field ──────────────────────────────────────────────────────────

    #[test]
    fn parse_field_simple() {
        assert_eq!(
            parse_field("user=me").ok(),
            Some(("user".to_string(), "me".to_string()))
        );
    }

    #[test]
    fn parse_field_no_equals_is_err() {
        assert!(parse_field("bad").is_err());
    }

    #[test]
    fn parse_field_empty_key_is_err() {
        assert!(parse_field("=v").is_err());
    }

    #[test]
    fn parse_field_value_with_equals_splits_on_first() {
        assert_eq!(
            parse_field("k=v=w").ok(),
            Some(("k".to_string(), "v=w".to_string()))
        );
    }

    // ── strip_one_trailing_newline ────────────────────────────────────────────

    #[test]
    fn strip_trailing_newline_lf() {
        assert_eq!(strip_one_trailing_newline("pw\n".into()), "pw");
    }

    #[test]
    fn strip_trailing_newline_crlf() {
        assert_eq!(strip_one_trailing_newline("pw\r\n".into()), "pw");
    }

    #[test]
    fn strip_trailing_newline_none() {
        assert_eq!(strip_one_trailing_newline("pw".into()), "pw");
    }

    #[test]
    fn strip_trailing_newline_multiline_strips_only_last() {
        assert_eq!(strip_one_trailing_newline("a\nb\n".into()), "a\nb");
    }

    // ── apply_set ────────────────────────────────────────────────────────────

    #[test]
    fn apply_set_preserves_otp_and_tags() {
        let mut entry = Entry::parse("oldpass\nuser: alice\notpauth://totp/x?secret=ABC\n@work\n");
        let fields = vec![("email".to_string(), "a@b.com".to_string())];
        apply_set(&mut entry, Some("newpass"), &fields, &[], &[], &[]);

        assert_eq!(entry.password(), "newpass", "password updated");
        assert_eq!(entry.field("email"), Some("a@b.com"), "new field present");
        assert_eq!(entry.field("user"), Some("alice"), "existing field intact");
        assert!(entry.otp_uri().is_some(), "OTP URI must be preserved");
        assert!(
            entry.tags().iter().any(|t| t == "work"),
            "work tag must be preserved"
        );
    }

    #[test]
    fn apply_set_no_password_preserves_existing() {
        let mut entry = Entry::parse("existing\nuser: bob\n");
        apply_set(
            &mut entry,
            None,
            &[("url".to_string(), "x.com".to_string())],
            &[],
            &[],
            &[],
        );
        assert_eq!(entry.password(), "existing", "password unchanged");
        assert_eq!(entry.field("url"), Some("x.com"));
    }

    // ── apply_set tag/field removal ──────────────────────────────────────────

    #[test]
    fn apply_set_adds_tag_dedups_and_preserves() {
        // Entry has @work already; add "work" + "dev" — "work" must not duplicate.
        let mut entry = Entry::parse("pw\nuser: alice\notpauth://totp/x?secret=ABC\n@work\n");
        apply_set(
            &mut entry,
            None,
            &[],
            &[],
            &["work".to_string(), "dev".to_string()],
            &[],
        );

        let tags = entry.tags();
        assert_eq!(
            tags.iter().filter(|t| t.as_str() == "work").count(),
            1,
            "work must appear exactly once, got {tags:?}"
        );
        assert!(tags.iter().any(|t| t == "dev"), "dev tag must be present");
        assert_eq!(entry.password(), "pw", "password preserved");
        assert_eq!(entry.field("user"), Some("alice"), "user field preserved");
        assert!(entry.otp_uri().is_some(), "OTP preserved");
    }

    #[test]
    fn apply_set_removes_field_only_target() {
        let mut entry = Entry::parse("pw\nuser: alice\notpauth://totp/x?secret=ABC\n@work\n");
        apply_set(
            &mut entry,
            None,
            &[("url".to_string(), "https://x.com".to_string())],
            &["user".to_string()],
            &[],
            &[],
        );

        assert_eq!(entry.field("user"), None, "user field must be removed");
        assert_eq!(entry.field("url"), Some("https://x.com"), "url field added");
        assert!(entry.otp_uri().is_some(), "OTP intact");
        assert!(entry.tags().iter().any(|t| t == "work"), "work tag intact");
        assert_eq!(entry.password(), "pw", "password intact");
    }

    #[test]
    fn apply_set_removes_tag_only_target() {
        let mut entry = Entry::parse("pw\nuser: bob\n@work @dev\n");
        apply_set(&mut entry, None, &[], &[], &[], &["dev".to_string()]);

        let tags = entry.tags();
        assert!(tags.iter().any(|t| t == "work"), "work must remain");
        assert!(!tags.iter().any(|t| t == "dev"), "dev must be removed");
        assert_eq!(entry.field("user"), Some("bob"), "user field intact");
        assert_eq!(entry.password(), "pw", "password intact");
    }

    #[test]
    fn apply_set_no_tag_change_leaves_tags_untouched() {
        // Add only a field (no tag ops): set_tags must NOT fire, so the raw
        // bytes — including the @work @dev tag line — stay byte-identical except
        // for the appended field. Capturing serialize() before/after a no-tag
        // call proves the guard fires (tags() alone is a recomputed view).
        let mut entry = Entry::parse("pw\nuser: bob\n@work @dev\n");
        // Add the field first, then assert the tag line is byte-stable across a
        // second call that touches nothing tag-related.
        entry.set_field("url", "x.com");
        let before = entry.serialize();
        apply_set(&mut entry, None, &[], &[], &[], &[]);
        assert_eq!(
            entry.serialize(),
            before,
            "raw bytes unchanged when no tag ops"
        );
    }

    /// Entry with password, a user field, an otp URI, and a @work tag.
    fn entry_full() -> Entry {
        Entry::parse("s3cr3t\nuser: alice\notpauth://totp/x?secret=ABC\n@work\n")
    }

    /// Entry with password and a user field only (no otp, no tags).
    fn entry_minimal() -> Entry {
        Entry::parse("s3cr3t\nuser: alice\n")
    }

    // ── show_json ────────────────────────────────────────────────────────────

    #[test]
    fn show_json_includes_path_fields_tags_has_otp() {
        let e = entry_full();
        let json_str = show_json(&e, "web/github.com");
        let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");

        assert_eq!(v["path"], "web/github.com");
        assert_eq!(v["has_otp"], true);
        let fields = v["fields"].as_array().expect("fields is array");
        assert!(
            fields.iter().any(|f| f[0] == "user" && f[1] == "alice"),
            "user field present"
        );
        let tags = v["tags"].as_array().expect("tags is array");
        assert!(tags.iter().any(|t| t == "work"), "work tag present");
    }

    #[test]
    fn show_json_does_not_contain_password_or_otp_uri() {
        let e = entry_full();
        let json_str = show_json(&e, "web/github.com");

        // Must not contain the actual password text.
        assert!(
            !json_str.contains("s3cr3t"),
            "password must not appear in JSON output"
        );
        // Must not contain the raw otpauth URI.
        assert!(
            !json_str.contains("otpauth://"),
            "otpauth URI must not appear in JSON output"
        );
    }

    #[test]
    fn show_json_no_otp_has_otp_false() {
        let e = entry_minimal();
        let json_str = show_json(&e, "email/work");
        let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");

        assert_eq!(v["has_otp"], false);
        assert_eq!(v["tags"].as_array().unwrap().len(), 0, "no tags");
    }

    // ── show_plain ───────────────────────────────────────────────────────────

    #[test]
    fn show_plain_includes_field_and_otp_yes() {
        let e = entry_full();
        let text = show_plain(&e, "web/github.com");

        assert!(text.contains("user: alice"), "user field present");
        assert!(text.contains("otp: yes"), "otp line present");
        assert!(text.contains("work"), "tag present");
    }

    #[test]
    fn show_plain_does_not_contain_password_or_otp_uri() {
        let e = entry_full();
        let text = show_plain(&e, "web/github.com");

        assert!(
            !text.contains("s3cr3t"),
            "password must not appear in plain output"
        );
        assert!(
            !text.contains("otpauth://"),
            "otpauth URI must not appear in plain output"
        );
    }

    #[test]
    fn show_plain_no_otp_shows_otp_no() {
        let e = entry_minimal();
        let text = show_plain(&e, "email/work");

        assert!(text.contains("otp: no"), "otp: no present");
        assert!(!text.contains("tags:"), "no tags line when empty");
    }

    #[test]
    fn show_plain_starts_with_path() {
        let e = entry_full();
        let text = show_plain(&e, "web/github.com");
        assert!(text.starts_with("web/github.com\n"), "path is first line");
    }
}
