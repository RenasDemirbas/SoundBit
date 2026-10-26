//! SoundBit desktop app (composition root).
//!
//! Wires the shared editing session to the Svelte front end, live audio playback and an
//! embedded localhost MCP endpoint, so AI agents edit the same song the user sees.

#![forbid(unsafe_code)]
// Hide the console window of release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod dto;
mod state;
mod sync;

use tauri::Manager;

use crate::state::AppState;

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let workspace = state::workspace_dir(app.handle())?;
            app.manage(AppState::start(workspace));
            sync::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::state::get_state,
            commands::state::get_environment,
            commands::state::edit,
            commands::state::undo,
            commands::state::redo,
            commands::files::new_song,
            commands::files::open_project,
            commands::files::save_project,
            commands::files::export_wav,
            commands::playback::play,
            commands::playback::stop,
            commands::playback::playback_state,
            commands::presets::list_presets,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("soundbit: {error}");
        std::process::exit(1);
    }
}
