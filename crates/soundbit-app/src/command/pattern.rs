//! Pattern and note commands.

use soundbit_core::{InstrumentId, ModelError, Note, Pattern, PatternId, Pitch, Song};

use crate::CommandError;
use crate::edit::Edit;

/// Identifies a note within a pattern: no two notes share a start step and pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteKey {
    /// Start step.
    pub start: u16,
    /// Pitch.
    pub pitch: Pitch,
}

/// Changes to a pattern's properties. `None` keeps the current value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PatternChanges {
    /// New name.
    pub name: Option<String>,
    /// New instrument.
    pub instrument: Option<InstrumentId>,
}

/// A change to the song's patterns.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternCommand {
    /// Add a pattern (with or without notes).
    Create(Pattern),
    /// Rename a pattern or change its instrument.
    Update(PatternId, PatternChanges),
    /// Replace all notes of a pattern.
    SetNotes(PatternId, Vec<Note>),
    /// Add notes to a pattern. Fails if one overlaps an existing note of the same pitch.
    AddNotes(PatternId, Vec<Note>),
    /// Remove notes from a pattern. Fails if one of them does not exist.
    RemoveNotes(PatternId, Vec<NoteKey>),
    /// Move every note of a pattern by a number of semitones.
    Transpose(PatternId, i32),
    /// Copy a pattern under a new id.
    Duplicate {
        /// The pattern to copy.
        source: PatternId,
        /// Name of the copy; defaults to the original name plus " copy".
        name: Option<String>,
    },
    /// Remove a pattern that is not placed in the arrangement.
    Remove(PatternId),
}

impl PatternCommand {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Create(pattern) => format!("Create pattern \"{}\"", pattern.name),
            Self::Update(id, _) => format!("Edit {id}"),
            Self::SetNotes(id, _) => format!("Write notes of {id}"),
            Self::AddNotes(id, notes) => format!("Add {} note(s) to {id}", notes.len()),
            Self::RemoveNotes(id, keys) => format!("Remove {} note(s) from {id}", keys.len()),
            Self::Transpose(id, semitones) => format!("Transpose {id} by {semitones:+} semitones"),
            Self::Duplicate { source, .. } => format!("Duplicate {source}"),
            Self::Remove(id) => format!("Remove {id}"),
        }
    }

    pub(crate) fn into_edit(self, song: &Song) -> Result<Edit, CommandError> {
        Ok(match self {
            Self::Create(pattern) => Edit::AddPattern(pattern),
            Self::Update(id, changes) => {
                let mut pattern = current(song, id)?;
                if let Some(name) = changes.name {
                    pattern.name = name;
                }
                if let Some(instrument) = changes.instrument {
                    pattern.instrument = instrument;
                }
                Edit::ReplacePattern(id, pattern)
            }
            Self::SetNotes(id, notes) => {
                let mut pattern = current(song, id)?;
                pattern.set_notes(notes)?;
                Edit::ReplacePattern(id, pattern)
            }
            Self::AddNotes(id, notes) => {
                let mut pattern = current(song, id)?;
                for note in notes {
                    pattern.insert_note(note)?;
                }
                Edit::ReplacePattern(id, pattern)
            }
            Self::RemoveNotes(id, keys) => {
                Edit::ReplacePattern(id, without_notes(song, id, &keys)?)
            }
            Self::Transpose(id, semitones) => {
                Edit::ReplacePattern(id, transposed(song, id, semitones)?)
            }
            Self::Duplicate { source, name } => {
                let mut copy = current(song, source)?;
                copy.name = name.unwrap_or_else(|| format!("{} copy", copy.name));
                Edit::AddPattern(copy)
            }
            Self::Remove(id) => Edit::RemovePattern(id),
        })
    }
}

/// A copy of the pattern with this id.
fn current(song: &Song, id: PatternId) -> Result<Pattern, ModelError> {
    song.pattern(id)
        .cloned()
        .ok_or(ModelError::UnknownPattern(id))
}

fn without_notes(song: &Song, id: PatternId, keys: &[NoteKey]) -> Result<Pattern, CommandError> {
    let mut pattern = current(song, id)?;
    for key in keys {
        if pattern.remove_note(key.start, key.pitch).is_none() {
            return Err(CommandError::NoteNotFound {
                pattern: id,
                start: key.start,
                pitch: key.pitch,
            });
        }
    }
    Ok(pattern)
}

fn transposed(song: &Song, id: PatternId, semitones: i32) -> Result<Pattern, ModelError> {
    let mut pattern = current(song, id)?;
    let notes = pattern
        .notes()
        .iter()
        .map(|note| {
            Ok(Note {
                pitch: note.pitch.transpose(semitones)?,
                ..*note
            })
        })
        .collect::<Result<Vec<_>, ModelError>>()?;
    pattern.set_notes(notes)?;
    Ok(pattern)
}
