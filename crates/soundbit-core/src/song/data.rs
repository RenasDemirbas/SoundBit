//! Validated deserialization of [`Song`].
//!
//! Deserialized data is checked with the same rules the editing methods enforce, so a loaded
//! song is always as consistent as one built in memory.

use serde::Deserialize;

use super::{Song, SongMeta};
use crate::arrangement::Track;
use crate::collection::Collection;
use crate::limits::{MAX_INSTRUMENTS, MAX_PATTERNS, MAX_TRACKS};
use crate::{
    ChipProfile, Instrument, InstrumentId, LoopRange, ModelError, Pattern, PatternId, Reverb,
    Scale, TimeSettings, TrackId,
};

#[derive(Deserialize)]
pub(super) struct SongData {
    #[serde(default)]
    meta: SongMeta,
    #[serde(default)]
    timing: TimeSettings,
    #[serde(default)]
    scale: Scale,
    #[serde(default)]
    chip_profile: ChipProfile,
    #[serde(default)]
    reverb: Reverb,
    #[serde(default = "Collection::new")]
    instruments: Collection<InstrumentId, Instrument>,
    #[serde(default = "Collection::new")]
    patterns: Collection<PatternId, Pattern>,
    #[serde(default = "Collection::new")]
    tracks: Collection<TrackId, Track>,
    #[serde(default)]
    loop_range: Option<LoopRange>,
}

impl TryFrom<SongData> for Song {
    type Error = ModelError;

    fn try_from(data: SongData) -> Result<Self, Self::Error> {
        let song = Self {
            meta: data.meta,
            timing: data.timing,
            scale: data.scale,
            chip_profile: data.chip_profile,
            reverb: data.reverb,
            instruments: data.instruments,
            patterns: data.patterns,
            tracks: data.tracks,
            loop_range: data.loop_range,
        };
        song.validate()?;
        Ok(song)
    }
}

impl Song {
    fn validate(&self) -> Result<(), ModelError> {
        check_size(self.instruments.len(), MAX_INSTRUMENTS, "instruments")?;
        check_size(self.patterns.len(), MAX_PATTERNS, "patterns")?;
        check_size(self.tracks.len(), MAX_TRACKS, "tracks")?;
        for (_, pattern) in self.patterns.iter() {
            self.check_pattern(pattern)?;
        }
        for (_, track) in self.tracks.iter() {
            self.check_track(track)?;
        }
        Ok(())
    }
}

fn check_size(len: usize, max: usize, what: &'static str) -> Result<(), ModelError> {
    if len <= max {
        Ok(())
    } else {
        Err(ModelError::LimitReached { what, max })
    }
}
