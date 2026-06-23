//! Shared backend state for Tauri commands.

use std::sync::{Mutex, MutexGuard};

use passcore::{Config, PasswordStore};

use crate::error::{not_initialized, CommandResult};

pub struct AppState {
    store: Option<Mutex<Box<dyn PasswordStore + Send>>>,
    pub init_error: Option<String>,
    pub demo: bool,
    pub config: Config,
}

impl AppState {
    /// Build from a concrete store (used by `main` and by tests with `FakeStore`).
    pub fn new(store: Box<dyn PasswordStore + Send>, config: Config) -> Self {
        Self {
            store: Some(Mutex::new(store)),
            init_error: None,
            demo: false,
            config,
        }
    }

    /// Build from a concrete store in demo mode.
    pub fn new_demo(store: Box<dyn PasswordStore + Send>, config: Config) -> Self {
        Self {
            store: Some(Mutex::new(store)),
            init_error: None,
            demo: true,
            config,
        }
    }

    /// Build with no store — the app launched but the runtime environment was
    /// not ready (missing `pass`, `gpg`, or store directory).  Commands will
    /// refuse to operate and return a `not_initialized` error.
    pub fn uninitialized(init_error: String, config: Config) -> Self {
        Self {
            store: None,
            init_error: Some(init_error),
            demo: false,
            config,
        }
    }

    /// Acquire the store lock, or return a `not_initialized` command error when
    /// no store was built at startup.  Recovers from a poisoned mutex instead
    /// of propagating the panic (a poisoned mutex still contains usable data).
    pub fn store(&self) -> CommandResult<MutexGuard<'_, Box<dyn PasswordStore + Send>>> {
        match &self.store {
            None => Err(not_initialized()),
            Some(m) => Ok(m.lock().unwrap_or_else(|p| p.into_inner())),
        }
    }
}
