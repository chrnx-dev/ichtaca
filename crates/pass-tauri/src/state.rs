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
}
