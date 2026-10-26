//! Song-wide tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use soundbit_app::command::{MetaChanges, ReverbChanges, SongCommand, TimingChanges};
use soundbit_app::model::{ChipProfile, LoopRange, PitchClass, Scale, ScaleKind, Song};
use soundbit_app::{Command, Session};

use super::{ToolResult, attempt, refuse, reply};
use crate::SoundBitServer;
use crate::input::{Choice, optional};
use crate::view::song_view;

/// Changes to song-wide settings. Only the fields you set are changed.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SongUpdateInput {
    /// Song title.
    title: Option<String>,
    /// Author or artist.
    author: Option<String>,
    /// Free-form description.
    description: Option<String>,
    /// Tempo in beats per minute, 20-400.
    bpm: Option<u16>,
    /// Grid resolution: steps per beat, 1-16 (4 = sixteenth notes). Fails if existing notes
    /// would no longer fit in a bar.
    steps_per_beat: Option<u8>,
    /// Beats per bar, 1-16. Fails if existing notes would no longer fit in a bar.
    beats_per_bar: Option<u8>,
    /// Swing in percent, 0-100.
    swing: Option<u8>,
    /// Key root, e.g. "A", "F#", "Bb".
    scale_root: Option<String>,
    /// Scale kind.
    scale_kind: Option<Choice<ScaleKind>>,
    /// Hardware style that limits tracks and voices; "free" removes limits.
    chip_profile: Option<Choice<ChipProfile>>,
    /// First bar of the loop region (set together with `loop_end`).
    loop_start: Option<u16>,
    /// Bar after the last looped bar.
    loop_end: Option<u16>,
    /// Remove the loop region.
    #[serde(default)]
    clear_loop: bool,
    /// Size of the shared reverb room in percent, 0-100: 20 is a small room, 60 a hall, 90+ a
    /// cathedral. Instruments join the room through their `reverb_send_percent`.
    reverb_size: Option<u8>,
    /// Reverb high-frequency damping in percent, 0-100: low is bright stone, high is soft.
    reverb_damping: Option<u8>,
}

impl SongUpdateInput {
    fn into_commands(self, song: &Song) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();
        if self.title.is_some() || self.author.is_some() || self.description.is_some() {
            let changes = MetaChanges {
                title: self.title,
                author: self.author,
                description: self.description,
            };
            commands.push(SongCommand::UpdateMeta(changes).into());
        }
        let timing = TimingChanges {
            bpm: optional(self.bpm)?,
            steps_per_beat: optional(self.steps_per_beat)?,
            beats_per_bar: optional(self.beats_per_bar)?,
            swing: optional(self.swing)?,
        };
        if timing != TimingChanges::default() {
            commands.push(SongCommand::UpdateTiming(timing).into());
        }
        if self.scale_root.is_some() || self.scale_kind.is_some() {
            let current = song.scale();
            let root = match &self.scale_root {
                Some(name) => name
                    .parse::<PitchClass>()
                    .map_err(|error| error.to_string())?,
                None => current.root,
            };
            let kind = self.scale_kind.map_or(current.kind, |kind| kind.0);
            commands.push(SongCommand::SetScale(Scale::new(root, kind)).into());
        }
        if let Some(profile) = self.chip_profile {
            commands.push(SongCommand::SetChipProfile(profile.0).into());
        }
        let reverb = ReverbChanges {
            size: optional(self.reverb_size)?,
            damping: optional(self.reverb_damping)?,
        };
        if reverb != ReverbChanges::default() {
            commands.push(SongCommand::UpdateReverb(reverb).into());
        }
        match (self.loop_start, self.loop_end, self.clear_loop) {
            (None, None, false) => {}
            (None, None, true) => commands.push(SongCommand::SetLoopRange(None).into()),
            (Some(start), Some(end), false) => {
                let range = LoopRange::new(start, end).map_err(|error| error.to_string())?;
                commands.push(SongCommand::SetLoopRange(Some(range)).into());
            }
            _ => {
                return Err(
                    "set `loop_start` and `loop_end` together, or only `clear_loop`".into(),
                );
            }
        }
        Ok(commands)
    }
}

/// Settings for a new song.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SongNewInput {
    /// Song title.
    title: String,
    /// Hardware style; defaults to "free".
    chip_profile: Option<Choice<ChipProfile>>,
    /// Tempo in beats per minute, 20-400. Defaults to 120.
    bpm: Option<u16>,
    /// Confirm discarding unsaved changes to the current song.
    #[serde(default)]
    discard_unsaved: bool,
}

#[tool_router(router = song_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Returns the whole song: settings, instruments, patterns (with note \
                       counts), tracks with their clips, an arrangement grid (rows = tracks, \
                       columns = bars, cells = pattern ids), chip-profile problems and undo state.",
        annotations(read_only_hint = true)
    )]
    fn song_get(&self) -> ToolResult {
        reply(&song_view(&self.session(), self.workspace()))
    }

    #[tool(
        description = "Changes song-wide settings: title, author, tempo, grid, swing, key and \
                          scale, chip profile, loop region or reverb room. Only the given fields \
                          change; all changes are one undo step."
    )]
    fn song_update(&self, Parameters(input): Parameters<SongUpdateInput>) -> ToolResult {
        let commands = attempt!(input.into_commands(self.session().song()));
        if commands.is_empty() {
            return refuse("nothing to change: set at least one field");
        }
        let command = Command::Batch {
            label: "Update song settings".to_owned(),
            commands,
        };
        self.apply(command, |session, _| song_view(session, self.workspace()))
    }

    #[tool(
        description = "Replaces the open song with a new, empty one. This clears the undo \
                       history. Fails if the current song has unsaved changes unless \
                       `discard_unsaved` is true; save it first with song_save.",
        annotations(destructive_hint = true)
    )]
    fn song_new(&self, Parameters(input): Parameters<SongNewInput>) -> ToolResult {
        let mut session = self.session();
        if session.is_modified() && !input.discard_unsaved {
            return refuse(
                "the current song has unsaved changes: call song_save first, or pass \
                           `discard_unsaved: true`",
            );
        }
        let mut song = Song::new(input.title);
        song.set_chip_profile(
            input
                .chip_profile
                .map_or(ChipProfile::Free, |profile| profile.0),
        );
        if let Some(bpm) = attempt!(optional(input.bpm)) {
            let timing = soundbit_app::model::TimeSettings {
                bpm,
                ..song.timing()
            };
            attempt!(song.set_timing(timing));
        }
        *session = Session::new(song);
        reply(&song_view(&session, self.workspace()))
    }
}
