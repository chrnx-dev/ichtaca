//! pass-tauri — Tauri 2 backend exposing passcore operations as commands.
//!
//! `run()` is called from `main.rs`. It builds the store (detects a real
//! `PassCliStore`, falling back to `FakeStore` so the app still launches when
//! `pass`/`gpg` are absent), wires up `AppState`, and registers every command.

pub mod commands;
pub mod error;
pub mod state;

use passcore::{Config, FakeStore, PassCliStore, PasswordStore};
use state::AppState;

/// Entry point shared by the desktop binary (and, when enabled, mobile).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load().unwrap_or_default();

    let store: Box<dyn PasswordStore + Send> = match PassCliStore::detect(config.store_dir.clone())
    {
        Ok(s) => Box::new(s),
        Err(_) => Box::new(FakeStore::new()),
    };

    let app_state = AppState::new(store, config);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::read::list,
            commands::read::show_meta,
            commands::read::reveal_password,
            commands::read::reveal_otp_uri,
            commands::read::otp_code,
            commands::read::search_fuzzy,
            commands::write::insert,
            commands::write::update_entry,
            commands::write::remove,
            commands::write::mv,
            commands::write::cp,
            commands::write::generate,
            commands::write::generate_password,
            commands::clipboard::copy_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running pass-tauri");
}
