//! Pattern and note tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use soundbit_app::command::{PatternChanges, PatternCommand};
use soundbit_app::model::{InstrumentId, Pattern, PatternId};
use soundbit_app::{Created, Outcome, Session};

use super::{ToolResult, attempt, created, refuse, reply};
use crate::SoundBitServer;
use crate::input::{NoteInput, NoteRef, notes};
use crate::view::pattern_view;

/// A new pattern: one bar of notes for one instrument.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternCreateInput {
    /// Display name, e.g. "Verse melody".
    name: String,
    /// Instrument that plays the notes.
    instrument_id: u32,
    /// Notes; may be empty. Notes must fit in one bar (`start + length <= steps_per_bar`).
    #[serde(default)]
    notes: Vec<NoteInput>,
}

/// A pattern to read, remove or duplicate.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternIdInput {
    /// Pattern id.
    id: u32,
}

/// Changes to a pattern's name or instrument.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternUpdateInput {
    /// Pattern id.
    id: u32,
    /// New name.
    name: Option<String>,
    /// New instrument.
    instrument_id: Option<u32>,
}

/// Notes for a pattern.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternNotesInput {
    /// Pattern id.
    id: u32,
    /// The notes.
    notes: Vec<NoteInput>,
}

/// Notes to remove from a pattern.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternRemoveNotesInput {
    /// Pattern id.
    id: u32,
    /// Notes to remove, identified by name and start step.
    notes: Vec<NoteRef>,
}

/// A transposition.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternTransposeInput {
    /// Pattern id.
    id: u32,
    /// Semitones to move every note by; negative moves down, 12 is one octave.
    semitones: i32,
}

/// A pattern copy.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PatternDuplicateInput {
    /// Pattern to copy.
    id: u32,
    /// Name of the copy. Defaults to the original name plus " copy".
    name: Option<String>,
}

fn show(id: PatternId) -> impl FnOnce(&Session, &Outcome) -> Value {
    move |session, _| pattern_view(session.song(), id)
}

fn show_created(session: &Session, outcome: &Outcome) -> Value {
    let pick = |c: &Created| {
        if let Created::Pattern(id) = c {
            Some(*id)
        } else {
            None
        }
    };
    created(outcome, pick).map_or(Value::Null, |id| pattern_view(session.song(), id))
}

#[tool_router(router = pattern_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Creates a pattern (one bar of notes for one instrument) and returns it \
                          with a tracker grid. Place it on a track with arrangement_place."
    )]
    fn pattern_create(&self, Parameters(input): Parameters<PatternCreateInput>) -> ToolResult {
        let mut pattern = Pattern::new(input.name, InstrumentId::new(input.instrument_id));
        attempt!(pattern.set_notes(attempt!(notes(&input.notes))));
        self.apply(PatternCommand::Create(pattern), show_created)
    }

    #[tool(
        description = "Returns a pattern's notes, a tracker grid (one row per step; note names \
                       where notes start, '|' while held, '.' when silent; chords use several \
                       columns) and where it is placed.",
        annotations(read_only_hint = true)
    )]
    fn pattern_get(&self, Parameters(input): Parameters<PatternIdInput>) -> ToolResult {
        let session = self.session();
        match pattern_view(session.song(), PatternId::new(input.id)) {
            Value::Null => refuse(format!("pattern {} does not exist", input.id)),
            view => reply(&view),
        }
    }

    #[tool(description = "Renames a pattern or changes the instrument that plays it.")]
    fn pattern_update(&self, Parameters(input): Parameters<PatternUpdateInput>) -> ToolResult {
        let id = PatternId::new(input.id);
        let changes = PatternChanges {
            name: input.name,
            instrument: input.instrument_id.map(InstrumentId::new),
        };
        self.apply(PatternCommand::Update(id, changes), show(id))
    }

    #[tool(
        description = "Replaces ALL notes of a pattern with the given ones. Use it to write or \
                          rewrite a whole bar in one call."
    )]
    fn pattern_set_notes(&self, Parameters(input): Parameters<PatternNotesInput>) -> ToolResult {
        let id = PatternId::new(input.id);
        self.apply(
            PatternCommand::SetNotes(id, attempt!(notes(&input.notes))),
            show(id),
        )
    }

    #[tool(
        description = "Adds notes to a pattern, keeping the existing ones. Fails if a note \
                          overlaps an existing note of the same pitch."
    )]
    fn pattern_add_notes(&self, Parameters(input): Parameters<PatternNotesInput>) -> ToolResult {
        let id = PatternId::new(input.id);
        self.apply(
            PatternCommand::AddNotes(id, attempt!(notes(&input.notes))),
            show(id),
        )
    }

    #[tool(description = "Removes notes from a pattern, identified by note name and start step.")]
    fn pattern_remove_notes(
        &self,
        Parameters(input): Parameters<PatternRemoveNotesInput>,
    ) -> ToolResult {
        let id = PatternId::new(input.id);
        let keys = attempt!(
            input
                .notes
                .iter()
                .map(NoteRef::to_key)
                .collect::<Result<Vec<_>, _>>()
        );
        self.apply(PatternCommand::RemoveNotes(id, keys), show(id))
    }

    #[tool(description = "Moves every note of a pattern up or down by a number of semitones.")]
    fn pattern_transpose(
        &self,
        Parameters(input): Parameters<PatternTransposeInput>,
    ) -> ToolResult {
        let id = PatternId::new(input.id);
        self.apply(PatternCommand::Transpose(id, input.semitones), show(id))
    }

    #[tool(description = "Copies a pattern under a new id, e.g. to make a variation of it.")]
    fn pattern_duplicate(
        &self,
        Parameters(input): Parameters<PatternDuplicateInput>,
    ) -> ToolResult {
        let command = PatternCommand::Duplicate {
            source: PatternId::new(input.id),
            name: input.name,
        };
        self.apply(command, show_created)
    }

    #[tool(
        description = "Removes a pattern. Fails while it is placed in the arrangement; clear \
                       those bars first with arrangement_clear.",
        annotations(destructive_hint = true)
    )]
    fn pattern_remove(&self, Parameters(input): Parameters<PatternIdInput>) -> ToolResult {
        let id = PatternId::new(input.id);
        self.apply(
            PatternCommand::Remove(id),
            move |_, _| json!({"removed_pattern_id": id.get()}),
        )
    }
}
