//! Project files and audio export, confined to the workspace.

use std::error::Error;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use soundbit_app::{RenderOptions, Session};

use super::{ToolResult, attempt, refuse, reply};
use crate::SoundBitServer;
use crate::view::song_view;

/// Longest song an agent may export, in seconds. Stops runaway renders.
const MAX_EXPORT_SECONDS: f64 = 600.0;

const PROJECT_EXTENSION: &str = "sbproj";

/// Where to save.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SongSaveInput {
    /// Path relative to the workspace, e.g. `songs/theme.sbproj`. Defaults to the file the song
    /// was opened from or last saved to.
    path: Option<String>,
}

/// What to open.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SongOpenInput {
    /// Path relative to the workspace (see `files_list`).
    path: String,
    /// Confirm discarding unsaved changes to the current song.
    #[serde(default)]
    discard_unsaved: bool,
}

/// Where and how to export.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportWavInput {
    /// Path relative to the workspace, e.g. `renders/theme.wav`.
    path: String,
    /// Sample rate in hertz, 8000-192000. Defaults to 48000.
    sample_rate: Option<u32>,
}

/// An error and its causes as one sentence.
fn describe(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(next) = cause {
        message = format!("{message}: {next}");
        cause = next.source();
    }
    message
}

#[tool_router(router = file_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Lists SoundBit project files (.sbproj) in the workspace.",
        annotations(read_only_hint = true)
    )]
    fn files_list(&self) -> ToolResult {
        let workspace = self.workspace();
        reply(
            &json!({"workspace": workspace.root().display().to_string(), "projects": workspace.projects(PROJECT_EXTENSION)}),
        )
    }

    #[tool(description = "Saves the song as a project file in the workspace.")]
    fn song_save(&self, Parameters(input): Parameters<SongSaveInput>) -> ToolResult {
        let mut session = self.session();
        let workspace = self.workspace();
        let path = match (input.path, session.path()) {
            (Some(path), _) => attempt!(workspace.resolve(&path, PROJECT_EXTENSION)),
            (None, Some(current)) => current.to_owned(),
            (None, None) => return refuse("the song has never been saved: give a `path`"),
        };
        if let Err(error) = session.save_as(&path) {
            return refuse(describe(&error));
        }
        reply(&json!({"saved": workspace.display(&path)}))
    }

    #[tool(
        description = "Opens a project file from the workspace, replacing the open song and \
                       clearing the undo history. Fails if the current song has unsaved changes \
                       unless `discard_unsaved` is true.",
        annotations(destructive_hint = true)
    )]
    fn song_open(&self, Parameters(input): Parameters<SongOpenInput>) -> ToolResult {
        let mut session = self.session();
        if session.is_modified() && !input.discard_unsaved {
            return refuse(
                "the current song has unsaved changes: call song_save first, or pass \
                           `discard_unsaved: true`",
            );
        }
        let path = attempt!(self.workspace().resolve(&input.path, PROJECT_EXTENSION));
        match Session::open(&path) {
            Ok(opened) => *session = opened,
            Err(error) => return refuse(describe(&error)),
        }
        reply(&song_view(&session, self.workspace()))
    }

    #[tool(
        description = "Renders the song to a 16-bit stereo WAV file in the workspace so the \
                          user can listen to it. Rendering is deterministic. Reports the peak and \
                          RMS level of the mix and of every track, so you can balance the mix \
                          without hearing it."
    )]
    fn export_wav(&self, Parameters(input): Parameters<ExportWavInput>) -> ToolResult {
        let path = attempt!(self.workspace().resolve(&input.path, "wav"));
        let session = self.session();
        let seconds = session.song().length_seconds();
        if seconds == 0.0 {
            return refuse(
                "the song is empty: place patterns on tracks with arrangement_place first",
            );
        }
        if seconds > MAX_EXPORT_SECONDS {
            return refuse(format!(
                "the song is {seconds:.0} s long; exports are limited to {MAX_EXPORT_SECONDS:.0} s"
            ));
        }
        let options = RenderOptions {
            sample_rate: input
                .sample_rate
                .unwrap_or(RenderOptions::default().sample_rate),
            ..RenderOptions::default()
        };
        match session.export_wav(&path, &options) {
            Ok(report) => {
                let tracks: Vec<_> = session
                    .song()
                    .tracks()
                    .zip(&report.tracks)
                    .map(|((id, track), level)| {
                        json!({
                            "track_id": id.get(),
                            "name": track.settings().name,
                            "peak": rounded(level.peak),
                            "rms": rounded(level.rms),
                        })
                    })
                    .collect();
                reply(&json!({
                    "exported": self.workspace().display(&path),
                    "seconds": (report.duration_seconds * 10.0).round() / 10.0,
                    "peak": rounded(report.peak),
                    "rms": rounded(report.rms),
                    "tracks": tracks,
                }))
            }
            Err(error) => refuse(describe(&error)),
        }
    }
}

/// A level rounded to three decimals for compact reports.
fn rounded(level: f32) -> f64 {
    (f64::from(level) * 1_000.0).round() / 1_000.0
}
