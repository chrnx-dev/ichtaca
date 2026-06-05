//! Error type for all core operations.

use std::path::PathBuf;

/// Errors surfaced by `passcore`. Frontends translate these for display.
#[derive(Debug, thiserror::Error)]
pub enum PassError {
    #[error("the `pass` binary was not found on PATH")]
    PassNotInstalled,

    #[error("the `gpg` binary was not found on PATH")]
    GpgNotInstalled,

    #[error("password store not found at {0}")]
    StoreNotFound(PathBuf),

    #[error("entry not found: {0}")]
    EntryNotFound(String),

    #[error("entry already exists: {0}")]
    AlreadyExists(String),

    #[error("failed to decrypt entry {entry}: {message}")]
    DecryptFailed { entry: String, message: String },

    #[error("git operation failed: {0}")]
    GitError(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("could not parse entry: {0}")]
    Parse(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, PassError>;
