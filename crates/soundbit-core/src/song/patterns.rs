//! Pattern operations on [`Song`].

use super::{Song, check_capacity};
use crate::limits::{MAX_NOTES_PER_PATTERN, MAX_PATTERNS};
use crate::{ModelError, Pattern, PatternId, Removed};

impl Song {
    /// Patterns in display order.
    pub fn patterns(&self) -> impl Iterator<Item = (PatternId, &Pattern)> {
        self.patterns.iter()
    }

    /// The pattern with this id.
    pub fn pattern(&self, id: PatternId) -> Option<&Pattern> {
        self.patterns.get(id)
    }

    /// Adds a pattern and returns its new id.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::LimitReached`] if the song is full, or any error from
    /// [`Song::check_pattern`].
    pub fn add_pattern(&mut self, pattern: Pattern) -> Result<PatternId, ModelError> {
        check_capacity(self.patterns.len(), MAX_PATTERNS, "patterns")?;
        self.check_pattern(&pattern)?;
        Ok(self.patterns.push(pattern))
    }

    /// Replaces a pattern and returns the old one. This is how notes are edited: take a copy,
    /// change it, and store it back.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownPattern`] if there is no such pattern, or any error from
    /// [`Song::check_pattern`].
    pub fn replace_pattern(
        &mut self,
        id: PatternId,
        pattern: Pattern,
    ) -> Result<Pattern, ModelError> {
        if !self.patterns.contains(id) {
            return Err(ModelError::UnknownPattern(id));
        }
        self.check_pattern(&pattern)?;
        self.patterns
            .replace(id, pattern)
            .ok_or(ModelError::UnknownPattern(id))
    }

    /// Removes a pattern that is not placed in the arrangement.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownPattern`] if there is no such pattern and
    /// [`ModelError::PatternInUse`] if a track still plays it.
    pub fn remove_pattern(
        &mut self,
        id: PatternId,
    ) -> Result<Removed<PatternId, Pattern>, ModelError> {
        let usage = self
            .tracks
            .iter()
            .find_map(|(track, t)| t.first_use_of(id).map(|bar| (track, bar)));
        if let Some((track, bar)) = usage {
            return Err(ModelError::PatternInUse {
                pattern: id,
                track,
                bar,
            });
        }
        self.patterns
            .remove(id)
            .ok_or(ModelError::UnknownPattern(id))
    }

    /// Puts a removed pattern back under its old id and position.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::DuplicateId`] if the id is taken again,
    /// [`ModelError::LimitReached`] if the song is full, or any error from
    /// [`Song::check_pattern`].
    pub fn restore_pattern(
        &mut self,
        removed: Removed<PatternId, Pattern>,
    ) -> Result<(), ModelError> {
        check_capacity(self.patterns.len(), MAX_PATTERNS, "patterns")?;
        self.check_pattern(&removed.value)?;
        self.patterns
            .restore(removed)
            .map_err(|id| ModelError::DuplicateId(id.to_string()))
    }

    /// Checks that a pattern could be stored in this song as it is now.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownInstrument`] if its instrument does not exist,
    /// [`ModelError::LimitReached`] if it has too many notes, and
    /// [`ModelError::NoteOutsideBar`] if a note does not fit in one bar.
    pub fn check_pattern(&self, pattern: &Pattern) -> Result<(), ModelError> {
        if !self.instruments.contains(pattern.instrument) {
            return Err(ModelError::UnknownInstrument(pattern.instrument));
        }
        if pattern.notes().len() > MAX_NOTES_PER_PATTERN {
            return Err(ModelError::LimitReached {
                what: "notes per pattern",
                max: MAX_NOTES_PER_PATTERN,
            });
        }
        let steps_per_bar = self.timing.steps_per_bar();
        match pattern
            .notes()
            .iter()
            .find(|note| note.end() > u32::from(steps_per_bar))
        {
            Some(note) => Err(ModelError::NoteOutsideBar {
                start: note.start,
                length: note.length.get(),
                steps_per_bar,
            }),
            None => Ok(()),
        }
    }
}
