// Tauri on Windows requires a Windows subsystem (no console window).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pass_tauri_lib::run();
}
