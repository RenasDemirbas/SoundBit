//! Data sent to the front end. Mirrored by `src/lib/ipc/types.ts`.

use serde::Serialize;
use soundbit_app::Session;
use soundbit_app::model::{Pitch, Song};

use crate::state::AppState;

/// The whole editor state: the song in its project-file JSON shape plus derived information.
#[derive(Debug, Serialize)]
pub(crate) struct StateDto<'a> {
    editor_id: u64,
    revision: u64,
    song: &'a Song,
    steps_per_bar: u16,
    length_bars: u16,
    /// Arrangement length in seconds, without release tails.
    length_seconds: f64,
    /// Pitch classes (0 = C ... 11 = B) in the song's scale, for highlighting.
    scale_classes: Vec<u8>,
    /// Chip-profile channel name of each track, or `null` without a profile.
    channels: Vec<Option<&'static str>>,
    problems: Vec<String>,
    undo: Option<&'a str>,
    redo: Option<&'a str>,
    file: Option<String>,
    modified: bool,
}

impl<'a> StateDto<'a> {
    pub(crate) fn new(session: &'a Session) -> Self {
        let song = session.song();
        let editor = session.editor();
        let channels = (0..song.track_count())
            .map(|index| {
                song.chip_profile()
                    .channels()
                    .and_then(|channels| channels.get(index))
                    .map(|channel| channel.name)
            })
            .collect();
        Self {
            editor_id: editor.id(),
            revision: editor.revision(),
            song,
            steps_per_bar: song.timing().steps_per_bar(),
            length_bars: song.length_bars(),
            length_seconds: song.length_seconds(),
            scale_classes: (0..12)
                .filter(|&class| song.scale().contains(Pitch::saturating(60 + class)))
                .collect(),
            channels,
            problems: song
                .profile_violations()
                .iter()
                .map(ToString::to_string)
                .collect(),
            undo: editor.undo_label(),
            redo: editor.redo_label(),
            file: session.path().map(|path| path.display().to_string()),
            modified: session.is_modified(),
        }
    }
}

/// Audio device and MCP endpoint status.
#[derive(Debug, Serialize)]
pub(crate) struct EnvironmentDto {
    audio_device: Option<String>,
    audio_error: Option<String>,
    mcp_url: Option<String>,
    mcp_error: Option<String>,
    workspace: String,
}

impl EnvironmentDto {
    pub(crate) fn new(state: &AppState) -> Self {
        Self {
            audio_device: state.audio.device.clone(),
            audio_error: state.audio.error.clone(),
            mcp_url: state.mcp.url.clone(),
            mcp_error: state.mcp.error.clone(),
            workspace: state.workspace.display().to_string(),
        }
    }
}

/// Transport state for the play head.
#[derive(Debug, Serialize)]
pub(crate) struct PlaybackDto {
    pub(crate) playing: bool,
    pub(crate) position_bars: Option<f64>,
    pub(crate) position_seconds: Option<f64>,
}
