//! Tauri commands invoked by the front end. Each one locks the session briefly, runs an app
//! command or query, and returns plain data. Errors are returned as user-facing strings.

// Tauri passes state and arguments by value.
#![allow(clippy::needless_pass_by_value)]

mod action;
pub(crate) mod files;
pub(crate) mod playback;
pub(crate) mod presets;
pub(crate) mod state;

use std::error::Error;

use serde_json::Value;
use soundbit_app::Session;

use crate::dto::StateDto;

/// Result type of every command.
pub(crate) type CommandResult<T> = Result<T, String>;

/// The editor state as JSON for the front end.
pub(crate) fn snapshot(session: &Session) -> CommandResult<Value> {
    serde_json::to_value(StateDto::new(session)).map_err(|error| error.to_string())
}

/// An error and its causes as one sentence.
pub(crate) fn describe(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(next) = cause {
        message = format!("{message}: {next}");
        cause = next.source();
    }
    message
}
