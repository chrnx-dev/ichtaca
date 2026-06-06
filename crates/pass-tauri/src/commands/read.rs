use serde::Serialize;
use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct EntryMeta {
    pub path: String,
    /// `key: value` fields excluding the password line and the otp uri.
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub has_otp: bool,
}

#[derive(Debug, Serialize)]
pub struct OtpCode {
    pub code: String,
    pub seconds: u64,
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 2
pub fn list(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 2
pub fn show_meta(state: State<'_, AppState>, path: String) -> CommandResult<EntryMeta> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 2
pub fn reveal_password(state: State<'_, AppState>, path: String) -> CommandResult<String> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 2
pub fn otp_code(state: State<'_, AppState>, path: String) -> CommandResult<OtpCode> {
    todo!()
}

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 2
pub fn search_fuzzy(state: State<'_, AppState>, query: String) -> CommandResult<Vec<String>> {
    todo!()
}
