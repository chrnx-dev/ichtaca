//! passcore — domain logic for a `pass` (passwordstore.org) client.
//!
//! No UI lives here. Frontends (TUI, Tauri) consume the `PasswordStore`
//! trait and the domain types re-exported below.

pub mod config;
pub mod entry;
pub mod error;
pub mod secret;
pub mod store;

// Re-exports are added in later tasks as the types come into existence.
// pub use error::PassError;
// pub use secret::Secret;
