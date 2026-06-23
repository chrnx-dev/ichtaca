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
        // Write commands land in Task 11.
        Command::Copy { .. } | Command::Generate { .. } | Command::Set { .. } => {
            Err(CliError::Usage("this command is not yet implemented".into()))
        }
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
    let otp_label = if entry.otp_uri().is_some() { "yes" } else { "no" };
    out.push_str(&format!("otp: {otp_label}\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::Entry;

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
