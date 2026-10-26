//! Edits requested by the front end, translated into app commands.

mod instrument;
mod note;

use serde::Deserialize;
use soundbit_app::Command;
use soundbit_app::command::{
    InstrumentCommand, MetaChanges, PatternChanges, PatternCommand, ReverbChanges, SongCommand,
    TimingChanges, TrackChanges, TrackCommand,
};
use soundbit_app::model::{
    ChipProfile, Instrument, InstrumentId, ModelError, Pattern, PatternId, PitchClass, Scale,
    ScaleKind, Song, TrackId, TrackSettings, Voice,
};

use instrument::InstrumentUpdate;
use note::NoteToggle;

/// One edit from the UI. Mirrored by `Action` in `src/lib/ipc/types.ts`.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Action {
    UpdateSong(SongSettings),
    AddInstrument {
        name: String,
        voice: Voice,
    },
    AddPreset {
        preset: String,
    },
    UpdateInstrument(InstrumentUpdate),
    RemoveInstrument {
        id: u32,
    },
    CreatePattern {
        name: String,
        instrument: u32,
    },
    UpdatePattern {
        id: u32,
        name: Option<String>,
        instrument: Option<u32>,
    },
    DuplicatePattern {
        id: u32,
    },
    RemovePattern {
        id: u32,
    },
    TransposePattern {
        id: u32,
        semitones: i32,
    },
    ToggleNote(NoteToggle),
    AddTrack {
        name: String,
    },
    UpdateTrack(TrackUpdate),
    RemoveTrack {
        id: u32,
    },
    /// Places a pattern on a bar, or clears the bar when `pattern` is `null`.
    SetClip {
        track: u32,
        bar: u16,
        pattern: Option<u32>,
    },
}

/// Song-wide settings to change; unset fields keep their value.
#[derive(Debug, Deserialize)]
pub(crate) struct SongSettings {
    title: Option<String>,
    bpm: Option<u16>,
    steps_per_beat: Option<u8>,
    beats_per_bar: Option<u8>,
    swing: Option<u8>,
    chip_profile: Option<ChipProfile>,
    scale_root: Option<u8>,
    scale_kind: Option<ScaleKind>,
    reverb_size: Option<u8>,
    reverb_damping: Option<u8>,
}

/// Track properties to change.
#[derive(Debug, Deserialize)]
pub(crate) struct TrackUpdate {
    id: u32,
    name: Option<String>,
    muted: Option<bool>,
    volume: Option<u8>,
}

pub(super) fn ranged<T, R>(value: Option<R>) -> Result<Option<T>, String>
where
    T: TryFrom<R, Error = ModelError>,
{
    value
        .map(T::try_from)
        .transpose()
        .map_err(|error| error.to_string())
}

impl Action {
    pub(crate) fn into_command(self, song: &Song) -> Result<Command, String> {
        let pattern = PatternId::new;
        Ok(match self {
            Self::UpdateSong(settings) => settings.into_command(song)?,
            Self::AddInstrument { name, voice } => {
                InstrumentCommand::Add(Instrument::new(name, voice)).into()
            }
            Self::AddPreset { preset } => instrument::add_preset(&preset)?,
            Self::UpdateInstrument(update) => update.into_command()?,
            Self::RemoveInstrument { id } => {
                InstrumentCommand::Remove(InstrumentId::new(id)).into()
            }
            Self::CreatePattern { name, instrument } => {
                PatternCommand::Create(Pattern::new(name, InstrumentId::new(instrument))).into()
            }
            Self::UpdatePattern {
                id,
                name,
                instrument,
            } => {
                let changes = PatternChanges {
                    name,
                    instrument: instrument.map(InstrumentId::new),
                };
                PatternCommand::Update(pattern(id), changes).into()
            }
            Self::DuplicatePattern { id } => PatternCommand::Duplicate {
                source: pattern(id),
                name: None,
            }
            .into(),
            Self::RemovePattern { id } => PatternCommand::Remove(pattern(id)).into(),
            Self::TransposePattern { id, semitones } => {
                PatternCommand::Transpose(pattern(id), semitones).into()
            }
            Self::ToggleNote(toggle) => toggle.into_command(song)?,
            Self::AddTrack { name } => TrackCommand::Add(TrackSettings::new(name)).into(),
            Self::UpdateTrack(update) => update.into_command()?,
            Self::RemoveTrack { id } => TrackCommand::Remove(TrackId::new(id)).into(),
            Self::SetClip {
                track,
                bar,
                pattern: Some(id),
            } => TrackCommand::PlaceClips {
                track: TrackId::new(track),
                pattern: pattern(id),
                bars: vec![bar],
            }
            .into(),
            Self::SetClip {
                track,
                bar,
                pattern: None,
            } => TrackCommand::ClearClips {
                track: TrackId::new(track),
                bars: vec![bar],
            }
            .into(),
        })
    }
}

impl SongSettings {
    fn into_command(self, song: &Song) -> Result<Command, String> {
        let mut commands: Vec<Command> = Vec::new();
        if self.title.is_some() {
            let changes = MetaChanges {
                title: self.title,
                ..MetaChanges::default()
            };
            commands.push(SongCommand::UpdateMeta(changes).into());
        }
        let timing = TimingChanges {
            bpm: ranged(self.bpm)?,
            steps_per_beat: ranged(self.steps_per_beat)?,
            beats_per_bar: ranged(self.beats_per_bar)?,
            swing: ranged(self.swing)?,
        };
        if timing != TimingChanges::default() {
            commands.push(SongCommand::UpdateTiming(timing).into());
        }
        if self.scale_root.is_some() || self.scale_kind.is_some() {
            let current = song.scale();
            let root = ranged::<PitchClass, _>(self.scale_root)?.unwrap_or(current.root);
            let scale = Scale::new(root, self.scale_kind.unwrap_or(current.kind));
            commands.push(SongCommand::SetScale(scale).into());
        }
        if let Some(profile) = self.chip_profile {
            commands.push(SongCommand::SetChipProfile(profile).into());
        }
        let reverb = ReverbChanges {
            size: ranged(self.reverb_size)?,
            damping: ranged(self.reverb_damping)?,
        };
        if reverb != ReverbChanges::default() {
            commands.push(SongCommand::UpdateReverb(reverb).into());
        }
        Ok(Command::Batch {
            label: "Change song settings".to_owned(),
            commands,
        })
    }
}

impl TrackUpdate {
    fn into_command(self) -> Result<Command, String> {
        let changes = TrackChanges {
            name: self.name,
            muted: self.muted,
            volume: ranged(self.volume)?,
        };
        Ok(TrackCommand::Update(TrackId::new(self.id), changes).into())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "tests panic on unexpected errors by design"
    )]

    use super::*;

    #[test]
    fn actions_deserialize_from_front_end_json() {
        let json = r#"{"kind": "update_song", "bpm": 150, "chip_profile": "nes"}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(matches!(
            action,
            Action::UpdateSong(SongSettings { bpm: Some(150), .. })
        ));
        let json = r#"{"kind": "add_instrument", "name": "Lead", "voice": {"type": "pulse", "duty": "quarter"}}"#;
        assert!(serde_json::from_str::<Action>(json).is_ok());
    }

    #[test]
    fn instrument_groups_and_presets_deserialize() {
        let json = r#"{"kind": "update_instrument", "id": 1,
            "filter": {"mode": "low_pass", "cutoff": 900, "resonance": 20, "envelope_depth": 12},
            "unison": {"voices": 3, "detune": 14}, "reverb_send": 40}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(action.into_command(&Song::new("t")).is_ok());
        let json = r#"{"kind": "update_instrument", "id": 1, "unison": {"voices": 9}}"#;
        assert!(
            serde_json::from_str::<Action>(json).is_err(),
            "out of range"
        );
        let json = r#"{"kind": "add_preset", "preset": "kick"}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(action.into_command(&Song::new("t")).is_ok());
    }
}
