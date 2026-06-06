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
