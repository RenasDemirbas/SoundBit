//! Track and arrangement tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use soundbit_app::command::{TrackChanges, TrackCommand};
use soundbit_app::model::{PatternId, TrackId, TrackSettings};
use soundbit_app::{Command, Created, Outcome, Session};

use super::{ToolResult, attempt, created, refuse};
use crate::SoundBitServer;
use crate::input::{bar_selection, optional};
use crate::view::{arrangement_grid, track_view};

/// Most placements one `arrangement_place_many` call accepts.
const MAX_PLACEMENTS: usize = 512;

/// A new track.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct TrackAddInput {
    /// Display name, e.g. "Bass".
    name: String,
    /// Volume in percent, 0-100. Defaults to 80.
    volume: Option<u8>,
    /// Start muted.
    #[serde(default)]
    muted: bool,
}

/// Changes to a track.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct TrackUpdateInput {
    /// Track id.
    id: u32,
    /// New name.
    name: Option<String>,
    /// Volume in percent, 0-100.
    volume: Option<u8>,
    /// Mute or unmute.
    muted: Option<bool>,
}

/// A track to remove.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct TrackIdInput {
    /// Track id.
    id: u32,
}

/// Where to place a pattern. Give either `bars` or `start_bar` (+ `count`).
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ArrangementPlaceInput {
    /// Track to place on.
    track_id: u32,
    /// Pattern to place.
    pattern_id: u32,
    /// Explicit bars, counted from 0, e.g. [0, 2, 4].
    bars: Option<Vec<u16>>,
    /// First bar of a run of consecutive bars.
    start_bar: Option<u16>,
    /// Number of consecutive bars starting at `start_bar`. Defaults to 1.
    count: Option<u16>,
}

/// One pattern placement inside `arrangement_place_many`. Give either `bars` or `start_bar`
/// (+ `count`).
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PlacementInput {
    /// Track to place on.
    track_id: u32,
    /// Pattern to place.
    pattern_id: u32,
    /// Explicit bars, counted from 0.
    bars: Option<Vec<u16>>,
    /// First bar of a run of consecutive bars.
    start_bar: Option<u16>,
    /// Number of consecutive bars starting at `start_bar`. Defaults to 1.
    count: Option<u16>,
}

/// Many placements, applied in order as one undo step.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ArrangementPlaceManyInput {
    /// The placements, at most 512. Later ones replace earlier ones in the same bar.
    placements: Vec<PlacementInput>,
}

impl PlacementInput {
    fn into_command(self) -> Result<Command, String> {
        let bars = bar_selection(self.bars, self.start_bar, self.count)?;
        Ok(TrackCommand::PlaceClips {
            track: TrackId::new(self.track_id),
            pattern: PatternId::new(self.pattern_id),
            bars,
        }
        .into())
    }
}

/// Which bars to clear. Give either `bars` or `start_bar` (+ `count`).
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ArrangementClearInput {
    /// Track to clear.
    track_id: u32,
    /// Explicit bars, counted from 0.
    bars: Option<Vec<u16>>,
    /// First bar of a run of consecutive bars.
    start_bar: Option<u16>,
    /// Number of consecutive bars starting at `start_bar`. Defaults to 1.
    count: Option<u16>,
}

fn show(id: TrackId) -> impl FnOnce(&Session, &Outcome) -> Value {
    move |session, _| track_view(session.song(), id)
}

#[tool_router(router = track_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Adds an empty track at the end. With a chip profile, track N plays on \
                          the profile's channel N (see reference_get)."
    )]
    fn track_add(&self, Parameters(input): Parameters<TrackAddInput>) -> ToolResult {
        let mut settings = TrackSettings::new(input.name);
        settings.muted = input.muted;
        settings.volume = attempt!(optional(input.volume)).unwrap_or(settings.volume);
        self.apply(TrackCommand::Add(settings), |session, outcome| {
            let pick = |c: &Created| {
                if let Created::Track(id) = c {
                    Some(*id)
                } else {
                    None
                }
            };
            created(outcome, pick).map_or(Value::Null, |id| track_view(session.song(), id))
        })
    }

    #[tool(description = "Renames, mutes/unmutes or changes the volume of a track.")]
    fn track_update(&self, Parameters(input): Parameters<TrackUpdateInput>) -> ToolResult {
        let id = TrackId::new(input.id);
        let changes = TrackChanges {
            name: input.name,
            muted: input.muted,
            volume: attempt!(optional(input.volume)),
        };
        self.apply(TrackCommand::Update(id, changes), show(id))
    }

    #[tool(
        description = "Removes a track together with its clips. Later tracks move up one \
                       chip-profile channel.",
        annotations(destructive_hint = true)
    )]
    fn track_remove(&self, Parameters(input): Parameters<TrackIdInput>) -> ToolResult {
        let id = TrackId::new(input.id);
        self.apply(
            TrackCommand::Remove(id),
            move |_, _| json!({"removed_track_id": id.get()}),
        )
    }

    #[tool(
        description = "Places a pattern on a track at one or more bars, replacing what was \
                          there. Example: start_bar 0, count 8 repeats it for eight bars."
    )]
    fn arrangement_place(
        &self,
        Parameters(input): Parameters<ArrangementPlaceInput>,
    ) -> ToolResult {
        let track = TrackId::new(input.track_id);
        let bars = attempt!(bar_selection(input.bars, input.start_bar, input.count));
        let pattern = PatternId::new(input.pattern_id);
        self.apply(
            TrackCommand::PlaceClips {
                track,
                pattern,
                bars,
            },
            show(track),
        )
    }

    #[tool(
        description = "Places many patterns at once, e.g. a whole arrangement, as one undo step. \
                       Each placement is like arrangement_place. If any placement is invalid, \
                       nothing changes. Returns the arrangement grid."
    )]
    fn arrangement_place_many(
        &self,
        Parameters(input): Parameters<ArrangementPlaceManyInput>,
    ) -> ToolResult {
        let count = input.placements.len();
        if count == 0 || count > MAX_PLACEMENTS {
            return refuse(format!(
                "give between 1 and {MAX_PLACEMENTS} placements, got {count}"
            ));
        }
        let mut commands = Vec::with_capacity(count);
        for (index, placement) in input.placements.into_iter().enumerate() {
            let command = placement
                .into_command()
                .map_err(|error| format!("placements[{index}]: {error}"));
            commands.push(attempt!(command));
        }
        let command = Command::Batch {
            label: format!("Place {count} clips"),
            commands,
        };
        self.apply(
            command,
            |session, _| json!({"arrangement": arrangement_grid(session.song())}),
        )
    }

    #[tool(description = "Clears one or more bars of a track.")]
    fn arrangement_clear(
        &self,
        Parameters(input): Parameters<ArrangementClearInput>,
    ) -> ToolResult {
        let track = TrackId::new(input.track_id);
        let bars = attempt!(bar_selection(input.bars, input.start_bar, input.count));
        self.apply(TrackCommand::ClearClips { track, bars }, show(track))
    }
}
