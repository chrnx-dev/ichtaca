use tauri::State;

use crate::error::CommandResult;
use crate::state::AppState;

#[tauri::command]
#[allow(unused_variables)] // body filled in Phase-1 Task 4
pub fn copy_password(state: State<'_, AppState>, path: String) -> CommandResult<()> {
    todo!()
}
