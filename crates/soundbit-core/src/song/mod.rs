//! The song aggregate: the only way to change a song while keeping it consistent.
//!
//! Every mutating method either applies its change completely or returns an error and leaves
//! the song untouched. Methods that replace or remove something return the previous value, so
//! the application layer can build exact undo operations.

mod data;
mod instruments;
mod patterns;
mod tracks;

use serde::{Deserialize, Serialize};

use crate::arrangement::Track;
use crate::chip::{self, ProfileViolation};
use crate::collection::Collection;
use crate::{
    ChipProfile, Instrument, InstrumentId, LoopRange, ModelError, Pattern, PatternId, Reverb,
    Scale, TimeSettings, TrackId,
};

/// Descriptive information about a song.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct SongMeta {
    /// Song title.
    pub title: String,
    /// Author or artist.
    pub author: String,
    /// Free-form notes.
    pub description: String,
}

/// A complete song: instruments, patterns, the arrangement of patterns on tracks, and global
/// settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "data::SongData")]
pub struct Song {
    meta: SongMeta,
    timing: TimeSettings,
    scale: Scale,
    chip_profile: ChipProfile,
    reverb: Reverb,
    instruments: Collection<InstrumentId, Instrument>,
    patterns: Collection<PatternId, Pattern>,
    tracks: Collection<TrackId, Track>,
    loop_range: Option<LoopRange>,
}

impl Default for Song {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

impl Song {
    /// Creates an empty song with default settings.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            meta: SongMeta {
                title: title.into(),
                ..SongMeta::default()
            },
            timing: TimeSettings::default(),
            scale: Scale::default(),
            chip_profile: ChipProfile::default(),
            reverb: Reverb::default(),
            instruments: Collection::new(),
            patterns: Collection::new(),
            tracks: Collection::new(),
            loop_range: None,
        }
    }

    /// Descriptive information.
    pub fn meta(&self) -> &SongMeta {
        &self.meta
    }

    /// Replaces the descriptive information and returns the old one.
    pub fn set_meta(&mut self, meta: SongMeta) -> SongMeta {
        std::mem::replace(&mut self.meta, meta)
    }

    /// Tempo and grid.
    pub fn timing(&self) -> TimeSettings {
        self.timing
    }

    /// Changes tempo and grid and returns the old settings.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::PatternDoesNotFit`] if a pattern has notes beyond the new bar
    /// length.
    pub fn set_timing(&mut self, timing: TimeSettings) -> Result<TimeSettings, ModelError> {
        let steps_per_bar = timing.steps_per_bar();
        if let Some((pattern, needed)) = self
            .patterns
            .iter()
            .map(|(id, pattern)| (id, pattern.end_step()))
            .find(|&(_, end)| end > u32::from(steps_per_bar))
        {
            return Err(ModelError::PatternDoesNotFit {
                pattern,
                needed,
                steps_per_bar,
            });
        }
        Ok(std::mem::replace(&mut self.timing, timing))
    }

    /// Key and scale used to guide note entry.
    pub fn scale(&self) -> Scale {
        self.scale
    }

    /// Changes the scale and returns the old one. Existing notes are not changed.
    pub fn set_scale(&mut self, scale: Scale) -> Scale {
        std::mem::replace(&mut self.scale, scale)
    }

    /// The chip profile the song targets.
    pub fn chip_profile(&self) -> ChipProfile {
        self.chip_profile
    }

    /// Changes the chip profile and returns the old one.
    ///
    /// This never fails: a song may temporarily break its profile's limits while it is being
    /// edited. Use [`Song::profile_violations`] to find out whether it does.
    pub fn set_chip_profile(&mut self, profile: ChipProfile) -> ChipProfile {
        std::mem::replace(&mut self.chip_profile, profile)
    }

    /// The shared reverb room.
    pub fn reverb(&self) -> Reverb {
        self.reverb
    }

    /// Changes the reverb room and returns the old settings.
    pub fn set_reverb(&mut self, reverb: Reverb) -> Reverb {
        std::mem::replace(&mut self.reverb, reverb)
    }

    /// Every way the song breaks the limits of its chip profile. Empty if it complies.
    pub fn profile_violations(&self) -> Vec<ProfileViolation> {
        chip::violations(self)
    }

    /// The loop region, if one is set.
    pub fn loop_range(&self) -> Option<LoopRange> {
        self.loop_range
    }

    /// Sets or clears the loop region and returns the old one.
    pub fn set_loop_range(&mut self, range: Option<LoopRange>) -> Option<LoopRange> {
        std::mem::replace(&mut self.loop_range, range)
    }

    /// Arrangement length in bars: the bar after the last clip on any track.
    pub fn length_bars(&self) -> u16 {
        self.tracks
            .iter()
            .map(|(_, track)| track.end_bar())
            .max()
            .unwrap_or(0)
    }

    /// Arrangement length in seconds, without release tails.
    pub fn length_seconds(&self) -> f64 {
        let steps = f64::from(self.length_bars()) * f64::from(self.timing.steps_per_bar());
        let steps_per_minute =
            f64::from(self.timing.bpm.get()) * f64::from(self.timing.steps_per_beat.get());
        steps * 60.0 / steps_per_minute
    }
}

/// Fails with [`ModelError::LimitReached`] if a collection is already full.
fn check_capacity(len: usize, max: usize, what: &'static str) -> Result<(), ModelError> {
    if len < max {
        Ok(())
    } else {
        Err(ModelError::LimitReached { what, max })
    }
}
