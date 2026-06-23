//! Serializable command error returned to the webview.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<passcore::PassError> for CommandError {
    fn from(e: passcore::PassError) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;

/// Convenience constructor for the "store not initialized" error returned when
/// the runtime store was never built (missing `pass`, `gpg`, or store dir).
pub fn not_initialized() -> CommandError {
    CommandError {
        message: "password store not initialized".to_string(),
    }
}
