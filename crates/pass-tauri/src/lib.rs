//! pass-tauri — Tauri 2 backend exposing passcore operations as commands.
//!
//! `run()` is called from `main.rs`. It attempts to build the store via
//! `passcore::init_store`. On failure the app still launches but with no store
//! and a recorded init error; commands refuse to operate and the frontend can
//! call `doctor` to render a setup screen.

pub mod commands;
pub mod error;
pub mod state;

use passcore::Config;
use state::AppState;

/// Entry point shared by the desktop binary (and, when enabled, mobile).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load().unwrap_or_default();

    let app_state = match passcore::init_store(&config) {
        Ok(init) => {
            if init.demo {
                AppState::new_demo(init.store, config)
            } else {
                AppState::new(init.store, config)
            }
        }
        Err(e) => {
            // Keep the app launchable; the frontend will show a setup screen
            // (via the `doctor` command). Log to stderr for terminal debugging.
            eprintln!("ichtaca-desktop: store unavailable: {e}");
            AppState::uninitialized(e.to_string(), config)
        }
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::read::list,
            commands::read::show_meta,
            commands::read::reveal_password,
            commands::read::reveal_otp_uri,
            commands::read::otp_code,
            commands::read::search_fuzzy,
            commands::read::search_deep,
            commands::write::insert,
            commands::write::update_entry,
            commands::write::remove,
            commands::write::mv,
            commands::write::cp,
            commands::write::generate,
            commands::write::generate_password,
            commands::clipboard::copy_password,
            commands::doctor::doctor,
            commands::git::git_status,
            commands::git::git_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running pass-tauri");
}
