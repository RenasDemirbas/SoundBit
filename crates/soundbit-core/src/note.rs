//! Notes placed on a pattern's step grid.

use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;
use crate::{ModelError, Pitch};

ranged! {
    /// How hard a note is played, as a MIDI velocity.
    Velocity(u8): 1..=127, default 100, "velocity"
}

/// A note in a pattern. Time is measured in steps from the start of the pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Note {
    /// Pitch to play.
    pub pitch: Pitch,
    /// First step of the note, counted from 0.
    pub start: u16,
    /// Duration in steps.
    pub length: NonZeroU16,
    /// Velocity.
    #[serde(default)]
    pub velocity: Velocity,
}

impl Note {
    /// Creates a note with the default velocity.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::ZeroLengthNote`] if `length` is 0.
    pub fn new(pitch: Pitch, start: u16, length: u16) -> Result<Self, ModelError> {
        let length = NonZeroU16::new(length).ok_or(ModelError::ZeroLengthNote)?;
        Ok(Self {
            pitch,
            start,
            length,
            velocity: Velocity::default(),
        })
    }

    /// Returns the note with a different velocity.
    #[must_use]
    pub const fn with_velocity(self, velocity: Velocity) -> Self {
        Self { velocity, ..self }
    }

    /// The step right after the note ends.
    pub fn end(&self) -> u32 {
        u32::from(self.start) + u32::from(self.length.get())
    }

    /// Whether the two notes sound at the same time at any step.
    pub fn overlaps_in_time(&self, other: &Self) -> bool {
        u32::from(self.start) < other.end() && u32::from(other.start) < self.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(start: u16, length: u16) -> Note {
        Note::new(Pitch::default(), start, length).expect("non-zero length")
    }

    #[test]
    fn rejects_zero_length() {
        assert_eq!(
            Note::new(Pitch::default(), 0, 0),
            Err(ModelError::ZeroLengthNote)
        );
    }

    #[test]
    fn adjacent_notes_do_not_overlap() {
        assert!(!note(0, 4).overlaps_in_time(&note(4, 4)));
        assert!(note(0, 5).overlaps_in_time(&note(4, 4)));
        assert!(note(2, 1).overlaps_in_time(&note(0, 8)));
    }
}
