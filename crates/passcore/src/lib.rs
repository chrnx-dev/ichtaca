//! passcore — domain logic for a `pass` (passwordstore.org) client.

pub mod clipboard;
pub mod config;
pub mod entry;
pub mod error;
pub mod generate;
pub mod otp;
pub mod search;
pub mod secret;
pub mod store;

pub use config::Config;
pub use entry::{Entry, Template};
pub use error::{PassError, Result};
pub use generate::generate_password;
pub use otp::{code_at, current, Otp};
pub use search::{deep, fuzzy_paths, PathHit};
pub use secret::Secret;
pub use store::cli::PassCliStore;
pub use store::fake::FakeStore;
pub use store::{EntryNode, PasswordStore};
