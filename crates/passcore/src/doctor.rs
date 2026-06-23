//! Environment diagnostics shared by the TUI, desktop, and CLI.
//!
//! Unlike `PassCliStore::detect`, this never short-circuits — it reports the
//! state of every prerequisite so a setup screen can show all that's missing.

use std::path::PathBuf;

use crate::store::cli::resolve_store_dir;

/// Snapshot of the runtime environment.
#[derive(Debug, Clone)]
pub struct Report {
    pub pass: bool,
    pub gpg: bool,
    pub store: bool,
    pub store_dir: PathBuf,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.pass && self.gpg && self.store
    }
}

/// Probe `pass`, `gpg`, and the resolved store directory. Pure I/O, no errors.
pub fn run(override_dir: Option<PathBuf>) -> Report {
    let store_dir = resolve_store_dir(override_dir, |k| std::env::var(k).ok());
    Report {
        pass: which::which("pass").is_ok(),
        gpg: which::which("gpg").is_ok(),
        store: store_dir.is_dir(),
        store_dir,
    }
}

/// Actionable setup guidance for a failing report. Empty string when `ok()`.
pub fn guidance(report: &Report) -> String {
    if report.ok() {
        return String::new();
    }
    let mut missing = Vec::new();
    if !report.pass {
        missing.push("pass command");
    }
    if !report.gpg {
        missing.push("gpg command");
    }
    if !report.store {
        missing.push("password store");
    }
    let install = if cfg!(target_os = "macos") {
        "  brew install pass gnupg".to_string()
    } else {
        "  # Debian/Ubuntu: sudo apt install pass gnupg\n  # Fedora: sudo dnf install pass gnupg\n  # Arch: sudo pacman -S pass gnupg".to_string()
    };
    format!(
        "Ichtaca could not open your password store.\n\n\
         Missing: {}\n\
         Expected store: {}\n\n\
         Install and initialize:\n\n{install}\n  gpg --full-generate-key\n  pass init <your-gpg-key-id>\n",
        missing.join(", "),
        report.store_dir.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn report(pass: bool, gpg: bool, store: bool) -> Report {
        Report { pass, gpg, store, store_dir: PathBuf::from("/tmp/store") }
    }

    #[test]
    fn ok_only_when_all_present() {
        assert!(report(true, true, true).ok());
        assert!(!report(false, true, true).ok());
        assert!(!report(true, true, false).ok());
        assert!(!report(false, false, false).ok());
    }

    #[test]
    fn guidance_names_missing_pieces() {
        let g = guidance(&report(false, true, false));
        assert!(g.contains("pass command"));
        // gpg is present in the fixture, so it must not be named as missing.
        // (The static "pass init <your-gpg-key-id>" line legitimately mentions
        // gpg, so we assert against the missing-piece label, not the substring.)
        assert!(!g.contains("gpg command"));
        assert!(g.contains("/tmp/store"));
        assert!(g.contains("pass init"));
    }

    #[test]
    fn guidance_empty_when_ok() {
        assert!(guidance(&report(true, true, true)).is_empty());
    }
}
