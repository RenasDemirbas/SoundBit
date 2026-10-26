//! Track and arrangement commands.

use soundbit_core::{ModelError, PatternId, Song, TrackId, TrackSettings, Volume};

use crate::CommandError;
use crate::edit::Edit;

/// Changes to a track's settings. `None` keeps the current value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackChanges {
    /// New name.
    pub name: Option<String>,
    /// Mute or unmute.
    pub muted: Option<bool>,
    /// New volume.
    pub volume: Option<Volume>,
}

impl TrackChanges {
    fn applied_to(self, settings: &TrackSettings) -> TrackSettings {
        TrackSettings {
            name: self.name.unwrap_or_else(|| settings.name.clone()),
            muted: self.muted.unwrap_or(settings.muted),
            volume: self.volume.unwrap_or(settings.volume),
        }
    }
}

/// A change to tracks or the arrangement.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackCommand {
    /// Add an empty track at the end.
    Add(TrackSettings),
    /// Change a track's settings.
    Update(TrackId, TrackChanges),
    /// Remove a track and its clips.
    Remove(TrackId),
    /// Place a pattern on several bars of a track, replacing what was there.
    PlaceClips {
        /// The track.
        track: TrackId,
        /// The pattern to place.
        pattern: PatternId,
        /// Bars to place it on.
        bars: Vec<u16>,
    },
    /// Clear several bars of a track.
    ClearClips {
        /// The track.
        track: TrackId,
        /// Bars to clear.
        bars: Vec<u16>,
    },
}

impl TrackCommand {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Add(settings) => format!("Add track \"{}\"", settings.name),
            Self::Update(id, _) => format!("Edit {id}"),
            Self::Remove(id) => format!("Remove {id}"),
            Self::PlaceClips {
                track,
                pattern,
                bars,
            } => {
                format!("Place {pattern} on {track} ({} bar(s))", bars.len())
            }
            Self::ClearClips { track, bars } => format!("Clear {} bar(s) of {track}", bars.len()),
        }
    }

    pub(crate) fn into_edit(self, song: &Song) -> Result<Edit, CommandError> {
        Ok(match self {
            Self::Add(settings) => Edit::AddTrack(settings),
            Self::Update(id, changes) => {
                let track = song.track(id).ok_or(ModelError::UnknownTrack(id))?;
                Edit::TrackSettings(id, changes.applied_to(track.settings()))
            }
            Self::Remove(id) => Edit::RemoveTrack(id),
            Self::PlaceClips {
                track,
                pattern,
                bars,
            } => Edit::Batch(
                bars.into_iter()
                    .map(|bar| Edit::Clip {
                        track,
                        bar,
                        pattern: Some(pattern),
                    })
                    .collect(),
            ),
            Self::ClearClips { track, bars } => Edit::Batch(
                bars.into_iter()
                    .map(|bar| Edit::Clip {
                        track,
                        bar,
                        pattern: None,
                    })
                    .collect(),
            ),
        })
    }
}
