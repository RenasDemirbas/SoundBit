//! Toggling notes by clicking pattern grid cells.

use serde::Deserialize;
use soundbit_app::Command;
use soundbit_app::command::{NoteKey, PatternCommand};
use soundbit_app::model::{ModelError, Note, PatternId, Pitch, Song};

/// A click on a pattern grid cell: adds a note there, or removes the note covering it.
#[derive(Debug, Deserialize)]
pub(crate) struct NoteToggle {
    pattern: u32,
    start: u16,
    pitch: u8,
    length: u16,
}
impl NoteToggle {
    /// Removes the note of this pitch covering the cell, or adds one starting there. A new note
    /// is shortened so it fits in the bar and ends before the next note of the same pitch.
    pub(super) fn into_command(self, song: &Song) -> Result<Command, String> {
        let id = PatternId::new(self.pattern);
        let pattern = song
            .pattern(id)
            .ok_or_else(|| ModelError::UnknownPattern(id).to_string())?;
        let pitch = Pitch::new(self.pitch).map_err(|error| error.to_string())?;
        let same_pitch = || {
            pattern
                .notes()
                .iter()
                .filter(move |note| note.pitch == pitch)
        };
        let step = u32::from(self.start);

        if let Some(note) =
            same_pitch().find(|note| u32::from(note.start) <= step && step < note.end())
        {
            return Ok(PatternCommand::RemoveNotes(
                id,
                vec![NoteKey {
                    start: note.start,
                    pitch,
                }],
            )
            .into());
        }
        let bar_end = song.timing().steps_per_bar();
        let next_start = same_pitch()
            .map(|note| note.start)
            .filter(|&start| start > self.start)
            .min()
            .unwrap_or(bar_end);
        let length = self
            .length
            .min(next_start.saturating_sub(self.start))
            .max(1);
        let note = Note::new(pitch, self.start, length).map_err(|error| error.to_string())?;
        Ok(PatternCommand::AddNotes(id, vec![note]).into())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "tests panic on unexpected errors by design"
    )]

    use super::*;
    use crate::commands::action::Action;
    use soundbit_app::model::{Instrument, Pattern, Voice};
    use soundbit_app::{Editor, Origin};

    fn editor() -> (Editor, u32) {
        let mut song = Song::new("t");
        let lead = song
            .add_instrument(Instrument::new("Lead", Voice::Sine))
            .expect("room");
        let pattern = song.add_pattern(Pattern::new("A", lead)).expect("valid");
        (Editor::new(song), pattern.get())
    }

    fn toggle(editor: &mut Editor, pattern: u32, start: u16, length: u16) {
        let action = Action::ToggleNote(NoteToggle {
            pattern,
            start,
            pitch: 60,
            length,
        });
        let command = action.into_command(editor.song()).expect("valid");
        editor.execute(command, Origin::User).expect("applies");
    }

    fn notes(editor: &Editor, pattern: u32) -> Vec<(u16, u16)> {
        let pattern = editor
            .song()
            .pattern(PatternId::new(pattern))
            .expect("exists");
        pattern
            .notes()
            .iter()
            .map(|n| (n.start, n.length.get()))
            .collect()
    }

    #[test]
    fn toggling_adds_then_removes_from_any_covered_cell() {
        let (mut editor, pattern) = editor();
        toggle(&mut editor, pattern, 4, 4);
        assert_eq!(notes(&editor, pattern), [(4, 4)]);
        toggle(&mut editor, pattern, 6, 1);
        assert_eq!(notes(&editor, pattern), []);
    }

    #[test]
    fn new_notes_shrink_to_fit() {
        let (mut editor, pattern) = editor();
        toggle(&mut editor, pattern, 8, 1);
        toggle(&mut editor, pattern, 6, 8);
        toggle(&mut editor, pattern, 14, 8);
        assert_eq!(notes(&editor, pattern), [(6, 2), (8, 1), (14, 2)]);
    }
}
