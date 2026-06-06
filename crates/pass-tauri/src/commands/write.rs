use serde::Deserialize;
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

/// Form payload from the UI. `password` is line 1; `fields` are `key: value`
/// rows; `otp` is an optional `otpauth://` line; `tags` join as `@a @b`.
#[derive(Debug, Deserialize)]
pub struct EntryInput {
    pub password: String,
    pub fields: Vec<(String, String)>,
    pub otp: Option<String>,
    pub tags: Vec<String>,
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn insert(
    state: State<'_, AppState>,
    path: String,
    input: EntryInput,
    overwrite: bool,
) -> CommandResult<()> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn update_entry(
    state: State<'_, AppState>,
    path: String,
    input: EntryInput,
) -> CommandResult<()> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn remove(state: State<'_, AppState>, path: String) -> CommandResult<()> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn mv(state: State<'_, AppState>, from: String, to: String) -> CommandResult<()> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn cp(state: State<'_, AppState>, from: String, to: String) -> CommandResult<()> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 3
pub fn generate(
    state: State<'_, AppState>,
    path: String,
    len: usize,
    symbols: bool,
) -> CommandResult<()> {
    todo!()
}
