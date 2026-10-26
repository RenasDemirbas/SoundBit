//! Song-wide settings.

use soundbit_core::{
    BeatsPerBar, Bpm, ChipProfile, LoopRange, Reverb, ReverbDamping, RoomSize, Scale, Song,
    SongMeta, StepsPerBeat, Swing, TimeSettings,
};

use crate::edit::Edit;

/// Changes to the song's descriptive information. `None` keeps the current value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaChanges {
    /// New title.
    pub title: Option<String>,
    /// New author.
    pub author: Option<String>,
    /// New description.
    pub description: Option<String>,
}

impl MetaChanges {
    fn applied_to(self, meta: &SongMeta) -> SongMeta {
        SongMeta {
            title: self.title.unwrap_or_else(|| meta.title.clone()),
            author: self.author.unwrap_or_else(|| meta.author.clone()),
            description: self.description.unwrap_or_else(|| meta.description.clone()),
        }
    }
}

/// Changes to tempo and grid. `None` keeps the current value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimingChanges {
    /// New tempo.
    pub bpm: Option<Bpm>,
    /// New grid resolution.
    pub steps_per_beat: Option<StepsPerBeat>,
    /// New bar length in beats.
    pub beats_per_bar: Option<BeatsPerBar>,
    /// New swing amount.
    pub swing: Option<Swing>,
}

impl TimingChanges {
    fn applied_to(self, timing: TimeSettings) -> TimeSettings {
        TimeSettings {
            bpm: self.bpm.unwrap_or(timing.bpm),
            steps_per_beat: self.steps_per_beat.unwrap_or(timing.steps_per_beat),
            beats_per_bar: self.beats_per_bar.unwrap_or(timing.beats_per_bar),
            swing: self.swing.unwrap_or(timing.swing),
        }
    }
}

/// Changes to the reverb room. `None` keeps the current value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReverbChanges {
    /// New room size.
    pub size: Option<RoomSize>,
    /// New damping.
    pub damping: Option<ReverbDamping>,
}

impl ReverbChanges {
    fn applied_to(self, reverb: Reverb) -> Reverb {
        Reverb {
            size: self.size.unwrap_or(reverb.size),
            damping: self.damping.unwrap_or(reverb.damping),
        }
    }
}

/// A change to song-wide settings.
#[derive(Debug, Clone, PartialEq)]
pub enum SongCommand {
    /// Change title, author or description.
    UpdateMeta(MetaChanges),
    /// Change tempo or grid. Fails if a pattern would no longer fit in a bar.
    UpdateTiming(TimingChanges),
    /// Change the key and scale.
    SetScale(Scale),
    /// Change the chip profile.
    SetChipProfile(ChipProfile),
    /// Set or clear the loop region.
    SetLoopRange(Option<LoopRange>),
    /// Change the reverb room.
    UpdateReverb(ReverbChanges),
}

impl SongCommand {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::UpdateMeta(_) => "Edit song details".to_owned(),
            Self::UpdateTiming(_) => "Change tempo and grid".to_owned(),
            Self::SetScale(scale) => format!("Set scale to {} {:?}", scale.root, scale.kind),
            Self::SetChipProfile(profile) => format!("Set chip profile to {}", profile.name()),
            Self::SetLoopRange(Some(range)) => {
                format!("Loop bars {} to {}", range.start(), range.end() - 1)
            }
            Self::SetLoopRange(None) => "Clear loop".to_owned(),
            Self::UpdateReverb(_) => "Change reverb".to_owned(),
        }
    }

    pub(crate) fn into_edit(self, song: &Song) -> Edit {
        match self {
            Self::UpdateMeta(changes) => Edit::Meta(changes.applied_to(song.meta())),
            Self::UpdateTiming(changes) => Edit::Timing(changes.applied_to(song.timing())),
            Self::SetScale(scale) => Edit::Scale(scale),
            Self::SetChipProfile(profile) => Edit::ChipProfile(profile),
            Self::SetLoopRange(range) => Edit::LoopRange(range),
            Self::UpdateReverb(changes) => Edit::Reverb(changes.applied_to(song.reverb())),
        }
    }
}
