//! Errors produced when a change would break a model invariant.

use thiserror::Error;

use crate::{InstrumentId, PatternId, Pitch, TrackId};

/// A change was rejected because it would leave the song in an invalid state.
///
/// Messages are written to be actionable for both humans and AI agents: they say what was
/// wrong and, where possible, how to fix it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ModelError {
    /// A numeric parameter is outside its allowed range.
    #[error("{name} must be between {min} and {max}, got {value}")]
    OutOfRange {
        /// Human-readable parameter name.
        name: &'static str,
        /// The rejected value.
        value: i64,
        /// Smallest allowed value.
        min: i64,
        /// Largest allowed value.
        max: i64,
    },

    /// A note or pitch-class name could not be parsed.
    #[error(
        "invalid note name `{0}`: expected a letter A-G, an optional `#` or `b`, \
         and an octave (for pitches), e.g. `C4`, `F#3` or `Bb2`"
    )]
    InvalidNoteName(String),

    /// A note has a length of zero steps.
    #[error("note length must be at least 1 step")]
    ZeroLengthNote,

    /// A note does not fit inside the bar.
    #[error(
        "note at step {start} with length {length} does not fit in a bar of {steps_per_bar} \
         steps (steps are numbered from 0)"
    )]
    NoteOutsideBar {
        /// Start step of the note.
        start: u16,
        /// Length of the note in steps.
        length: u16,
        /// Steps available in one bar.
        steps_per_bar: u16,
    },

    /// Two notes with the same pitch overlap in time within one pattern.
    #[error("note {pitch} at step {start} overlaps another {pitch} in the same pattern")]
    OverlappingNote {
        /// Pitch of the rejected note.
        pitch: Pitch,
        /// Start step of the rejected note.
        start: u16,
    },

    /// No instrument with this id exists.
    #[error("{0} does not exist")]
    UnknownInstrument(InstrumentId),

    /// No pattern with this id exists.
    #[error("{0} does not exist")]
    UnknownPattern(PatternId),

    /// No track with this id exists.
    #[error("{0} does not exist")]
    UnknownTrack(TrackId),

    /// An entity with this id already exists.
    #[error("{0} already exists")]
    DuplicateId(String),

    /// The instrument is still used by a pattern.
    #[error("{instrument} is used by {pattern}; change or remove that pattern first")]
    InstrumentInUse {
        /// The instrument that was going to be removed.
        instrument: InstrumentId,
        /// A pattern that uses it.
        pattern: PatternId,
    },

    /// The pattern is still placed in the arrangement.
    #[error("{pattern} is placed on {track} at bar {bar}; clear it from the arrangement first")]
    PatternInUse {
        /// The pattern that was going to be removed.
        pattern: PatternId,
        /// A track that plays it.
        track: TrackId,
        /// The bar where it is placed.
        bar: u16,
    },

    /// A bar index is beyond the maximum arrangement length.
    #[error("bar {bar} is out of range; bars are numbered from 0 and at most {max} are allowed")]
    BarOutOfRange {
        /// The rejected bar index.
        bar: u16,
        /// Maximum number of bars.
        max: u16,
    },

    /// A loop range does not start before it ends.
    #[error("loop start (bar {start}) must be before loop end (bar {end})")]
    InvalidLoopRange {
        /// First bar of the loop.
        start: u16,
        /// Bar after the last bar of the loop.
        end: u16,
    },

    /// New time settings would cut off notes of an existing pattern.
    #[error(
        "{pattern} has notes up to step {needed}, which does not fit in {steps_per_bar} steps \
         per bar; shorten or move those notes first"
    )]
    PatternDoesNotFit {
        /// The pattern that would no longer fit.
        pattern: PatternId,
        /// Steps the pattern needs.
        needed: u32,
        /// Steps per bar under the new time settings.
        steps_per_bar: u16,
    },

    /// A song-wide size limit was reached.
    #[error("a song can have at most {max} {what}")]
    LimitReached {
        /// What is limited, e.g. "tracks".
        what: &'static str,
        /// The limit.
        max: usize,
    },
}
