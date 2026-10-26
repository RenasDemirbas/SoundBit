//! Keeps the front end and the audio thread in step with the session.
//!
//! The session can change from the GUI or from an AI agent over MCP. A background thread
//! watches the editor's `(id, revision)`, sends edits to the audio thread while playing, and
//! emits `song-changed` so the front end reloads.

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

/// How often the session is checked for changes.
const POLL: Duration = Duration::from_millis(60);

/// Event emitted to the front end when the song changed.
pub(crate) const SONG_CHANGED: &str = "song-changed";

pub(crate) fn spawn(app: AppHandle) {
    let spawned = thread::Builder::new()
        .name("soundbit-sync".to_owned())
        .spawn(move || {
            let mut last = None;
            loop {
                thread::sleep(POLL);
                let state = app.state::<AppState>();
                let version = {
                    let session = state.session();
                    let editor = session.editor();
                    if let Some(playback) = state.playback().as_mut()
                        && let Err(error) = playback.sync(editor)
                    {
                        eprintln!("soundbit: live update failed: {error}");
                    }
                    (editor.id(), editor.revision())
                };
                if last != Some(version) {
                    last = Some(version);
                    let _ = app.emit(SONG_CHANGED, version.1);
                }
            }
        });
    if let Err(error) = spawned {
        eprintln!("soundbit: cannot start the sync thread: {error}");
    }
}
