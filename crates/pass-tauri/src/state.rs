//! Shared backend state for Tauri commands.

use std::sync::Mutex;

use passcore::{Config, PasswordStore};

pub struct AppState {
    pub store: Mutex<Box<dyn PasswordStore + Send>>,
    pub config: Config,
}

impl AppState {
    /// Build from a concrete store (used by `main` and by tests with `FakeStore`).
    pub fn new(store: Box<dyn PasswordStore + Send>, config: Config) -> Self {
        Self {
            store: Mutex::new(store),
            config,
        }
    }

    /// Acquire the store lock, recovering from a poisoned mutex instead of
    /// propagating the panic.  A mutex is poisoned when a thread panicked while
    /// holding the lock; the data is still usable, so we extract it with
    /// `into_inner()` rather than cascading panics to every subsequent caller.
    pub fn store(&self) -> std::sync::MutexGuard<'_, Box<dyn PasswordStore + Send>> {
        self.store.lock().unwrap_or_else(|p| p.into_inner())
    }
}
