//! Reading and editing the song.

use serde_json::Value;
use soundbit_app::Origin;
use tauri::State;

use super::action::Action;
use super::{CommandResult, snapshot};
use crate::dto::EnvironmentDto;
use crate::state::AppState;

#[tauri::command]
pub(crate) fn get_state(state: State<'_, AppState>) -> CommandResult<Value> {
    snapshot(&state.session())
}

#[tauri::command]
pub(crate) fn get_environment(state: State<'_, AppState>) -> EnvironmentDto {
    EnvironmentDto::new(&state)
}

#[tauri::command]
pub(crate) fn edit(state: State<'_, AppState>, action: Action) -> CommandResult<Value> {
    let mut session = state.session();
    let command = action.into_command(session.song())?;
    session
        .editor_mut()
        .execute(command, Origin::User)
        .map_err(|error| error.to_string())?;
    snapshot(&session)
}

#[tauri::command]
pub(crate) fn undo(state: State<'_, AppState>) -> CommandResult<Value> {
    let mut session = state.session();
    session
        .editor_mut()
        .undo()
        .map_err(|error| error.to_string())?;
    snapshot(&session)
}

#[tauri::command]
pub(crate) fn redo(state: State<'_, AppState>) -> CommandResult<Value> {
    let mut session = state.session();
    session
        .editor_mut()
        .redo()
        .map_err(|error| error.to_string())?;
    snapshot(&session)
}
