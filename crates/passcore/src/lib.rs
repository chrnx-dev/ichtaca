//! passcore — domain logic for a `pass` (passwordstore.org) client.

pub mod config;
pub mod entry;
pub mod error;
pub mod otp;
pub mod secret;
pub mod store;

pub use config::Config;
pub use entry::Entry;
pub use error::{PassError, Result};
pub use secret::Secret;
pub use store::cli::PassCliStore;
pub use store::fake::FakeStore;
pub use store::{EntryNode, PasswordStore};
