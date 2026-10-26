//! Tracks and the bar grid they form.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::limits::MAX_BARS;
use crate::{ModelError, PatternId, Volume};

/// User-editable properties of a track.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackSettings {
    /// Display name.
    pub name: String,
    /// Whether the track is silenced.
    #[serde(default)]
    pub muted: bool,
    /// Output volume.
    #[serde(default)]
    pub volume: Volume,
}

impl TrackSettings {
    /// Creates settings for an unmuted track at default volume.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            muted: false,
            volume: Volume::default(),
        }
    }
}

/// A lane of the arrangement. Each bar holds at most one pattern (a clip).
///
/// A track's position in the song decides which chip-profile channel it plays on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    #[serde(flatten)]
    settings: TrackSettings,
    #[serde(default, with = "clip_list")]
    clips: BTreeMap<u16, PatternId>,
}

impl Track {
    pub(crate) fn new(settings: TrackSettings) -> Self {
        Self {
            settings,
            clips: BTreeMap::new(),
        }
    }

    /// The track's settings.
    pub fn settings(&self) -> &TrackSettings {
        &self.settings
    }

    /// Placed patterns as `(bar, pattern)`, in bar order.
    pub fn clips(&self) -> impl Iterator<Item = (u16, PatternId)> + '_ {
        self.clips.iter().map(|(&bar, &pattern)| (bar, pattern))
    }

    /// The pattern placed at `bar`, if any.
    pub fn clip_at(&self, bar: u16) -> Option<PatternId> {
        self.clips.get(&bar).copied()
    }

    /// The bar right after the last clip, or 0 for an empty track.
    pub fn end_bar(&self) -> u16 {
        self.clips.last_key_value().map_or(0, |(&bar, _)| bar + 1)
    }

    pub(crate) fn replace_settings(&mut self, settings: TrackSettings) -> TrackSettings {
        std::mem::replace(&mut self.settings, settings)
    }

    pub(crate) fn place(&mut self, bar: u16, pattern: PatternId) -> Option<PatternId> {
        self.clips.insert(bar, pattern)
    }

    pub(crate) fn clear(&mut self, bar: u16) -> Option<PatternId> {
        self.clips.remove(&bar)
    }

    /// The first bar where `pattern` is placed.
    pub(crate) fn first_use_of(&self, pattern: PatternId) -> Option<u16> {
        self.clips()
            .find(|&(_, placed)| placed == pattern)
            .map(|(bar, _)| bar)
    }
}

/// Checks that `bar` is a valid arrangement position.
pub(crate) fn check_bar(bar: u16) -> Result<(), ModelError> {
    if bar < MAX_BARS {
        Ok(())
    } else {
        Err(ModelError::BarOutOfRange { bar, max: MAX_BARS })
    }
}

/// Clips are stored as a list of `{bar, pattern}` objects, which reads better than a map with
/// stringified keys and survives `#[serde(flatten)]`.
mod clip_list {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::PatternId;

    #[derive(Serialize, Deserialize)]
    struct Clip {
        bar: u16,
        pattern: PatternId,
    }

    pub(super) fn serialize<S: Serializer>(
        clips: &BTreeMap<u16, PatternId>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(clips.iter().map(|(&bar, &pattern)| Clip { bar, pattern }))
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<u16, PatternId>, D::Error> {
        let mut clips = BTreeMap::new();
        for clip in Vec::<Clip>::deserialize(deserializer)? {
            if clips.insert(clip.bar, clip.pattern).is_some() {
                let message = format!("two clips on the same track at bar {}", clip.bar);
                return Err(serde::de::Error::custom(message));
            }
        }
        Ok(clips)
    }
}

/// A loop region of the arrangement: bars `start` up to, but not including, `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "LoopRangeData")]
pub struct LoopRange {
    start: u16,
    end: u16,
}

#[derive(Deserialize)]
struct LoopRangeData {
    start: u16,
    end: u16,
}

impl TryFrom<LoopRangeData> for LoopRange {
    type Error = ModelError;

    fn try_from(data: LoopRangeData) -> Result<Self, Self::Error> {
        Self::new(data.start, data.end)
    }
}

impl LoopRange {
    /// Creates a loop range.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidLoopRange`] unless `start < end`, and
    /// [`ModelError::BarOutOfRange`] if `end` is past the maximum arrangement length.
    pub fn new(start: u16, end: u16) -> Result<Self, ModelError> {
        if start >= end {
            return Err(ModelError::InvalidLoopRange { start, end });
        }
        if end > MAX_BARS {
            return Err(ModelError::BarOutOfRange {
                bar: end,
                max: MAX_BARS,
            });
        }
        Ok(Self { start, end })
    }

    /// First bar of the loop.
    pub const fn start(self) -> u16 {
        self.start
    }

    /// Bar right after the last bar of the loop.
    pub const fn end(self) -> u16 {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_range_must_be_ordered_and_bounded() {
        assert!(LoopRange::new(0, 4).is_ok());
        assert_eq!(
            LoopRange::new(4, 4),
            Err(ModelError::InvalidLoopRange { start: 4, end: 4 })
        );
        assert!(LoopRange::new(0, MAX_BARS + 1).is_err());
    }

    #[test]
    fn end_bar_follows_last_clip() {
        let mut track = Track::new(TrackSettings::new("bass"));
        assert_eq!(track.end_bar(), 0);
        track.place(3, PatternId::new(1));
        track.place(1, PatternId::new(2));
        assert_eq!(track.end_bar(), 4);
        assert_eq!(track.first_use_of(PatternId::new(1)), Some(3));
    }
}
