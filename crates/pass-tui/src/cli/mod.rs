//! Non-interactive `ichtaca` CLI. Bare `ichtaca` launches the TUI; a subcommand
//! runs here and exits. Every command reuses `passcore` — no store/parse logic
//! is duplicated.

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ichtaca",
    version,
    about = "Pass-compatible password manager (TUI + CLI)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all entry paths.
    List,
    /// Fuzzy-search entry paths.
    Search { query: String },
    /// Print an entry's password to stdout.
    Get { path: String },
    /// Copy an entry's password to the clipboard.
    Copy { path: String },
    /// Print the current TOTP code for an entry.
    Otp { path: String },
    /// Generate a password and store it at PATH.
    Generate {
        path: String,
        #[arg(long, default_value_t = 32)]
        length: usize,
        #[arg(long)]
        no_symbols: bool,
    },
    /// Create/update an entry; password read from stdin with --password-stdin.
    /// Repeat --field/--tag/--remove-field/--remove-tag for multiple values.
    Set {
        path: String,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Add a tag (repeatable).
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Remove a field by key (repeatable).
        #[arg(long = "remove-field")]
        remove_fields: Vec<String>,
        /// Remove a tag (repeatable).
        #[arg(long = "remove-tag")]
        remove_tags: Vec<String>,
    },
    /// Show an entry's metadata (use --json for machine-readable output).
    Show {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Diagnose the environment (pass/gpg/store).
    Doctor,
}

/// CLI-layer error: either a passcore failure or a usage/input problem.
pub enum CliError {
    Pass(passcore::PassError),
    /// User/input error (bad argument, entry has no OTP, etc.) — exit code 1.
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Pass(e) => write!(f, "{e}"),
            CliError::Usage(m) => write!(f, "{m}"),
        }
    }
}

impl From<passcore::PassError> for CliError {
    fn from(e: passcore::PassError) -> Self {
        CliError::Pass(e)
    }
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Pass(e) => exit_code(e),
            CliError::Usage(_) => 1,
        }
    }
}

/// Map a `PassError` to a stable process exit code. Part of the CLI's public
/// contract — do not reshuffle without a version note.
/// 0 success · 1 user/input · 2 missing dependency/store · 3 store/decrypt failure.
pub fn exit_code(err: &passcore::PassError) -> i32 {
    use passcore::PassError::*;
    match err {
        PassNotInstalled | GpgNotInstalled | StoreNotFound(_) | Config(_) => 2,
        EntryNotFound(_) | AlreadyExists(_) | Parse(_) => 1,
        DecryptFailed { .. } | GitError(_) | Io(_) => 3,
    }
}

/// Run a subcommand, returning the process exit code.
pub fn dispatch(cmd: Command) -> i32 {
    match cmd {
        Command::Doctor => doctor(),
        other => match commands::run(other) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("ichtaca: {e}");
                e.exit_code()
            }
        },
    }
}

/// `ichtaca doctor` — print per-prerequisite status; exit 0 when ready, else 2.
fn doctor() -> i32 {
    let config = passcore::Config::load().unwrap_or_default();
    let report = passcore::doctor::run(config.store_dir.clone());
    let mark = |ok: bool| if ok { "ok" } else { "missing" };
    println!("pass:  {}", mark(report.pass));
    println!("gpg:   {}", mark(report.gpg));
    println!(
        "store: {} ({})",
        mark(report.store),
        report.store_dir.display()
    );
    if report.ok() {
        0
    } else {
        eprint!("\n{}", passcore::doctor::guidance(&report));
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::PassError;
    use std::path::PathBuf;

    #[test]
    fn exit_codes_are_stable_per_bucket() {
        assert_eq!(exit_code(&PassError::PassNotInstalled), 2);
        assert_eq!(exit_code(&PassError::StoreNotFound(PathBuf::from("/x"))), 2);
        assert_eq!(exit_code(&PassError::EntryNotFound("a".into())), 1);
        assert_eq!(exit_code(&PassError::AlreadyExists("a".into())), 1);
        assert_eq!(
            exit_code(&PassError::DecryptFailed {
                entry: "a".into(),
                message: "m".into()
            }),
            3
        );
        assert_eq!(exit_code(&PassError::GitError("g".into())), 3);
    }

    #[test]
    fn cli_parses_subcommands() {
        // clap's derive should accept the documented surface without panicking.
        use clap::Parser;
        let cli = Cli::try_parse_from(["ichtaca", "doctor"]).unwrap();
        assert!(matches!(cli.cmd, Some(Command::Doctor)));
        let none = Cli::try_parse_from(["ichtaca"]).unwrap();
        assert!(none.cmd.is_none());
    }
}
