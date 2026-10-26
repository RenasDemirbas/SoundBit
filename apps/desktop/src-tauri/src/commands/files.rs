//! New, open, save and export. Paths come from the user's file dialogs.

use std::path::PathBuf;

use serde_json::Value;
use soundbit_app::model::Song;
use soundbit_app::{RenderOptions, Session};
use tauri::State;

use super::{CommandResult, describe, snapshot};
use crate::state::AppState;

#[tauri::command]
pub(crate) fn new_song(state: State<'_, AppState>, title: String) -> CommandResult<Value> {
    let mut session = state.session();
    *session = Session::new(Song::new(title));
    snapshot(&session)
}

#[tauri::command]
pub(crate) fn open_project(state: State<'_, AppState>, path: PathBuf) -> CommandResult<Value> {
    let opened = Session::open(&path).map_err(|error| describe(&error))?;
    let mut session = state.session();
    *session = opened;
    snapshot(&session)
}

#[tauri::command]
pub(crate) fn save_project(
    state: State<'_, AppState>,
    path: Option<PathBuf>,
) -> CommandResult<Value> {
    let mut session = state.session();
    let path = path
        .or_else(|| session.path().map(ToOwned::to_owned))
        .ok_or("choose where to save the song")?;
    session.save_as(&path).map_err(|error| describe(&error))?;
    snapshot(&session)
}

#[tauri::command]
pub(crate) fn export_wav(state: State<'_, AppState>, path: PathBuf) -> CommandResult<String> {
    let session = state.session();
    let report = session
        .export_wav(&path, &RenderOptions::default())
        .map_err(|error| describe(&error))?;
    Ok(format!(
        "Exported {} ({:.1} s)",
        path.display(),
        report.duration_seconds
    ))
}
