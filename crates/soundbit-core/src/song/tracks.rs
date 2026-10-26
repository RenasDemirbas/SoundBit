//! Track and arrangement operations on [`Song`].

use super::{Song, check_capacity};
use crate::arrangement::{Track, check_bar};
use crate::limits::MAX_TRACKS;
use crate::{ModelError, PatternId, Removed, TrackId, TrackSettings};

impl Song {
    /// Tracks in arrangement order. Track *n* plays on chip-profile channel *n*.
    pub fn tracks(&self) -> impl Iterator<Item = (TrackId, &Track)> {
        self.tracks.iter()
    }

    /// The track with this id.
    pub fn track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(id)
    }

    /// Number of tracks.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Adds an empty track at the end and returns its new id.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::LimitReached`] if the song already has the maximum number of
    /// tracks.
    pub fn add_track(&mut self, settings: TrackSettings) -> Result<TrackId, ModelError> {
        check_capacity(self.tracks.len(), MAX_TRACKS, "tracks")?;
        Ok(self.tracks.push(Track::new(settings)))
    }

    /// Replaces a track's settings and returns the old ones. Its clips are kept.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownTrack`] if there is no such track.
    pub fn set_track_settings(
        &mut self,
        id: TrackId,
        settings: TrackSettings,
    ) -> Result<TrackSettings, ModelError> {
        let track = self
            .tracks
            .get_mut(id)
            .ok_or(ModelError::UnknownTrack(id))?;
        Ok(track.replace_settings(settings))
    }

    /// Removes a track together with its clips.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownTrack`] if there is no such track.
    pub fn remove_track(&mut self, id: TrackId) -> Result<Removed<TrackId, Track>, ModelError> {
        self.tracks.remove(id).ok_or(ModelError::UnknownTrack(id))
    }

    /// Puts a removed track back under its old id and position.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::DuplicateId`] if the id is taken again,
    /// [`ModelError::LimitReached`] if the song is full, and
    /// [`ModelError::UnknownPattern`] if one of its clips refers to a pattern that no longer
    /// exists.
    pub fn restore_track(&mut self, removed: Removed<TrackId, Track>) -> Result<(), ModelError> {
        check_capacity(self.tracks.len(), MAX_TRACKS, "tracks")?;
        self.check_track(&removed.value)?;
        self.tracks
            .restore(removed)
            .map_err(|id| ModelError::DuplicateId(id.to_string()))
    }

    /// Places a pattern on a track at a bar and returns the pattern that was there before.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownTrack`] or [`ModelError::UnknownPattern`] for unknown
    /// ids and [`ModelError::BarOutOfRange`] if the bar is past the maximum length.
    pub fn place_clip(
        &mut self,
        track: TrackId,
        bar: u16,
        pattern: PatternId,
    ) -> Result<Option<PatternId>, ModelError> {
        check_bar(bar)?;
        if !self.patterns.contains(pattern) {
            return Err(ModelError::UnknownPattern(pattern));
        }
        let lane = self
            .tracks
            .get_mut(track)
            .ok_or(ModelError::UnknownTrack(track))?;
        Ok(lane.place(bar, pattern))
    }

    /// Clears a bar of a track and returns the pattern that was there.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownTrack`] if there is no such track.
    pub fn clear_clip(
        &mut self,
        track: TrackId,
        bar: u16,
    ) -> Result<Option<PatternId>, ModelError> {
        let lane = self
            .tracks
            .get_mut(track)
            .ok_or(ModelError::UnknownTrack(track))?;
        Ok(lane.clear(bar))
    }

    /// Checks that every clip of a track refers to an existing pattern at a valid bar.
    pub(super) fn check_track(&self, track: &Track) -> Result<(), ModelError> {
        for (bar, pattern) in track.clips() {
            check_bar(bar)?;
            if !self.patterns.contains(pattern) {
                return Err(ModelError::UnknownPattern(pattern));
            }
        }
        Ok(())
    }
}
