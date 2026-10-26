//! Patterns: one bar of notes for one instrument.

use serde::{Deserialize, Serialize};

use crate::{InstrumentId, ModelError, Note, Pitch};

/// One bar of notes played by a single instrument.
///
/// Notes are kept sorted by start step, then pitch. Two notes with the same pitch never
/// overlap; notes with different pitches may (chords). Whether the notes fit in the bar and
/// whether the instrument exists is checked by [`Song`](crate::Song) when the pattern is
/// stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PatternData")]
pub struct Pattern {
    /// Display name.
    pub name: String,
    /// Instrument that plays the notes.
    pub instrument: InstrumentId,
    notes: Vec<Note>,
}

#[derive(Deserialize)]
struct PatternData {
    name: String,
    instrument: InstrumentId,
    #[serde(default)]
    notes: Vec<Note>,
}

impl TryFrom<PatternData> for Pattern {
    type Error = ModelError;

    fn try_from(data: PatternData) -> Result<Self, Self::Error> {
        let mut pattern = Self::new(data.name, data.instrument);
        pattern.set_notes(data.notes)?;
        Ok(pattern)
    }
}

impl Pattern {
    /// Creates an empty pattern.
    pub fn new(name: impl Into<String>, instrument: InstrumentId) -> Self {
        Self {
            name: name.into(),
            instrument,
            notes: Vec::new(),
        }
    }

    /// Notes sorted by start step, then pitch.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// Adds a note.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::OverlappingNote`] if a note with the same pitch overlaps it.
    pub fn insert_note(&mut self, note: Note) -> Result<(), ModelError> {
        insert_sorted(&mut self.notes, note)
    }

    /// Removes the note with this start step and pitch, if there is one.
    pub fn remove_note(&mut self, start: u16, pitch: Pitch) -> Option<Note> {
        let index = self
            .notes
            .iter()
            .position(|n| n.start == start && n.pitch == pitch)?;
        Some(self.notes.remove(index))
    }

    /// Replaces all notes at once and returns the previous ones. Nothing changes on error.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::OverlappingNote`] if two notes with the same pitch overlap.
    pub fn set_notes(&mut self, notes: Vec<Note>) -> Result<Vec<Note>, ModelError> {
        let mut staged = Vec::with_capacity(notes.len());
        for note in notes {
            insert_sorted(&mut staged, note)?;
        }
        Ok(std::mem::replace(&mut self.notes, staged))
    }

    /// Removes all notes and returns them.
    pub fn clear_notes(&mut self) -> Vec<Note> {
        std::mem::take(&mut self.notes)
    }

    /// The step right after the last note ends, or 0 for an empty pattern.
    pub fn end_step(&self) -> u32 {
        self.notes.iter().map(Note::end).max().unwrap_or(0)
    }

    /// The largest number of notes sounding at the same time.
    pub fn polyphony(&self) -> usize {
        let mut events: Vec<(u32, isize)> = self
            .notes
            .iter()
            .flat_map(|n| [(u32::from(n.start), 1), (n.end(), -1)])
            .collect();
        // At equal times, note-offs (-1) sort before note-ons, so touching notes don't stack.
        events.sort_unstable();
        let mut active: isize = 0;
        let mut peak: isize = 0;
        for (_, delta) in events {
            active += delta;
            peak = peak.max(active);
        }
        peak.unsigned_abs()
    }
}

fn insert_sorted(notes: &mut Vec<Note>, note: Note) -> Result<(), ModelError> {
    if notes
        .iter()
        .any(|n| n.pitch == note.pitch && n.overlaps_in_time(&note))
    {
        return Err(ModelError::OverlappingNote {
            pitch: note.pitch,
            start: note.start,
        });
    }
    let key = |n: &Note| (n.start, n.pitch);
    let index = notes.partition_point(|n| key(n) < key(&note));
    notes.insert(index, note);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(pitch: &str, start: u16, length: u16) -> Note {
        Note::new(pitch.parse().expect("valid pitch"), start, length).expect("valid note")
    }

    fn pattern() -> Pattern {
        Pattern::new("lead", InstrumentId::new(1))
    }

    #[test]
    fn keeps_notes_sorted() {
        let mut pattern = pattern();
        for n in [note("E4", 4, 1), note("C4", 0, 1), note("G4", 0, 1)] {
            pattern.insert_note(n).expect("no overlap");
        }
        let order: Vec<_> = pattern
            .notes()
            .iter()
            .map(|n| (n.start, n.pitch.to_string()))
            .collect();
        assert_eq!(
            order,
            [(0, "C4".into()), (0, "G4".into()), (4, "E4".into())]
        );
    }

    #[test]
    fn rejects_overlapping_same_pitch() {
        let mut pattern = pattern();
        pattern.insert_note(note("C4", 0, 4)).expect("first note");
        assert!(matches!(
            pattern.insert_note(note("C4", 2, 1)),
            Err(ModelError::OverlappingNote { start: 2, .. })
        ));
        pattern
            .insert_note(note("C4", 4, 1))
            .expect("touching notes are fine");
    }

    #[test]
    fn set_notes_is_atomic() {
        let mut pattern = pattern();
        pattern.insert_note(note("A4", 0, 1)).expect("first note");
        let result = pattern.set_notes(vec![note("C4", 0, 2), note("C4", 1, 1)]);
        assert!(result.is_err());
        assert_eq!(pattern.notes(), [note("A4", 0, 1)]);
    }

    #[test]
    fn measures_polyphony_and_end() {
        let mut pattern = pattern();
        assert_eq!((pattern.polyphony(), pattern.end_step()), (0, 0));
        for n in [
            note("C4", 0, 4),
            note("E4", 0, 4),
            note("G4", 2, 4),
            note("C5", 4, 2),
        ] {
            pattern.insert_note(n).expect("no overlap");
        }
        assert_eq!(pattern.polyphony(), 3);
        assert_eq!(pattern.end_step(), 6);
    }

    #[test]
    fn removes_by_start_and_pitch() {
        let mut pattern = pattern();
        pattern.insert_note(note("C4", 0, 1)).expect("first note");
        assert!(
            pattern
                .remove_note(0, "D4".parse().expect("valid"))
                .is_none()
        );
        assert!(
            pattern
                .remove_note(0, "C4".parse().expect("valid"))
                .is_some()
        );
        assert_eq!(pattern.notes(), []);
    }
}
