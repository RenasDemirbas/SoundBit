//! Commands: the user-level changes that GUI, MCP and CLI request.
//!
//! A command is planned against the current song, turned into a reversible primitive edit and
//! applied. [`Editor`](crate::Editor) runs commands on a draft copy, so a failing command never
//! leaves the song half-changed.

mod instrument;
mod pattern;
mod shaping;
mod song;
mod track;

pub use instrument::{EnvelopeChanges, InstrumentChanges, InstrumentCommand};
pub use pattern::{NoteKey, PatternChanges, PatternCommand};
pub use shaping::{FilterChanges, PitchEnvelopeChanges, UnisonChanges, VibratoChanges};
pub use song::{MetaChanges, ReverbChanges, SongCommand, TimingChanges};
pub use track::{TrackChanges, TrackCommand};

use soundbit_core::Song;

use crate::CommandError;
use crate::edit::{Applied, Edit};

/// A change to a song, applied atomically as one undo step.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Song-wide settings.
    Song(SongCommand),
    /// Instruments.
    Instrument(InstrumentCommand),
    /// Patterns and their notes.
    Pattern(PatternCommand),
    /// Tracks and the arrangement.
    Track(TrackCommand),
    /// Several commands applied in order as a single undo step. Later commands see the effects
    /// of earlier ones.
    Batch {
        /// Label shown in the undo history.
        label: String,
        /// The commands.
        commands: Vec<Command>,
    },
}

impl Command {
    /// Human-readable description for the undo history.
    pub fn label(&self) -> String {
        match self {
            Self::Song(command) => command.label(),
            Self::Instrument(command) => command.label(),
            Self::Pattern(command) => command.label(),
            Self::Track(command) => command.label(),
            Self::Batch { label, .. } => label.clone(),
        }
    }

    pub(crate) fn apply(self, song: &mut Song) -> Result<Applied, CommandError> {
        let edit = match self {
            Self::Song(command) => command.into_edit(song),
            Self::Instrument(command) => command.into_edit(song)?,
            Self::Pattern(command) => command.into_edit(song)?,
            Self::Track(command) => command.into_edit(song)?,
            Self::Batch { commands, .. } => return apply_all(commands, song),
        };
        Ok(edit.apply(song)?)
    }
}

fn apply_all(commands: Vec<Command>, song: &mut Song) -> Result<Applied, CommandError> {
    let mut inverses = Vec::with_capacity(commands.len());
    let mut created = Vec::new();
    for command in commands {
        let applied = command.apply(song)?;
        inverses.push(applied.inverse);
        created.extend(applied.created);
    }
    inverses.reverse();
    Ok(Applied {
        inverse: Edit::Batch(inverses),
        created,
    })
}

macro_rules! impl_from {
    ($($variant:ident($command:ty)),*) => {
        $(impl From<$command> for Command {
            fn from(command: $command) -> Self {
                Self::$variant(command)
            }
        })*
    };
}

impl_from!(
    Song(SongCommand),
    Instrument(InstrumentCommand),
    Pattern(PatternCommand),
    Track(TrackCommand)
);
