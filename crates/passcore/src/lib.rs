//! passcore — domain logic for a `pass` (passwordstore.org) client.

pub mod clipboard;
pub mod config;
pub mod doctor;
pub mod entry;
pub mod error;
pub mod generate;
pub mod git;
pub mod otp;
pub mod runtime;
pub mod search;
pub mod secret;
pub mod store;

pub use config::Config;
pub use entry::{Entry, Template};
pub use error::{PassError, Result};
pub use generate::generate_password;
pub use git::Status as GitStatus;
pub use otp::{code_at, current, Algorithm, Otp, OtpConfig};
pub use runtime::{init_store, StoreInit};
pub use search::{deep, fuzzy_paths, PathHit};
pub use secret::Secret;
pub use store::cli::{store_dir, PassCliStore};
pub use store::fake::FakeStore;
pub use store::{EntryNode, PasswordStore};
