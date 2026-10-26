//! Instrument edits from the instrument editor.

use serde::Deserialize;
use soundbit_app::command::{
    EnvelopeChanges, FilterChanges, InstrumentChanges, InstrumentCommand, PitchEnvelopeChanges,
    UnisonChanges, VibratoChanges,
};
use soundbit_app::model::{Envelope, Filter, InstrumentId, PitchEnvelope, Unison, Vibrato, Voice};
use soundbit_app::{Command, PresetLibrary};

use super::ranged;

/// Instrument properties to change. The editor sends whole groups (envelope, filter, ...),
/// which the domain types validate while deserializing.
#[derive(Debug, Deserialize)]
pub(crate) struct InstrumentUpdate {
    id: u32,
    name: Option<String>,
    voice: Option<Voice>,
    envelope: Option<Envelope>,
    pitch_envelope: Option<PitchEnvelope>,
    vibrato: Option<Vibrato>,
    unison: Option<Unison>,
    filter: Option<Filter>,
    volume: Option<u8>,
    pan: Option<i8>,
    reverb_send: Option<u8>,
}

impl InstrumentUpdate {
    pub(super) fn into_command(self) -> Result<Command, String> {
        let changes = InstrumentChanges {
            name: self.name,
            voice: self.voice,
            envelope: self.envelope.map(envelope).unwrap_or_default(),
            pitch_envelope: self.pitch_envelope.map(pitch_envelope).unwrap_or_default(),
            vibrato: self.vibrato.map(vibrato).unwrap_or_default(),
            unison: self.unison.map(unison).unwrap_or_default(),
            filter: self.filter.map(filter).unwrap_or_default(),
            volume: ranged(self.volume)?,
            pan: ranged(self.pan)?,
            reverb_send: ranged(self.reverb_send)?,
        };
        Ok(InstrumentCommand::Update(InstrumentId::new(self.id), changes).into())
    }
}

/// Adds a built-in preset as a new instrument.
pub(super) fn add_preset(id: &str) -> Result<Command, String> {
    let library = PresetLibrary::builtin().map_err(|error| error.to_string())?;
    let preset = library
        .find(id)
        .ok_or_else(|| format!("unknown preset \"{id}\""))?;
    Ok(InstrumentCommand::Add(preset.instrument.clone()).into())
}

fn envelope(value: Envelope) -> EnvelopeChanges {
    EnvelopeChanges {
        attack: Some(value.attack),
        decay: Some(value.decay),
        sustain: Some(value.sustain),
        release: Some(value.release),
    }
}

fn pitch_envelope(value: PitchEnvelope) -> PitchEnvelopeChanges {
    PitchEnvelopeChanges {
        semitones: Some(value.semitones),
        time: Some(value.time),
    }
}

fn vibrato(value: Vibrato) -> VibratoChanges {
    VibratoChanges {
        depth: Some(value.depth),
        rate: Some(value.rate),
        delay: Some(value.delay),
    }
}

fn unison(value: Unison) -> UnisonChanges {
    UnisonChanges {
        voices: Some(value.voices),
        detune: Some(value.detune),
    }
}

fn filter(value: Filter) -> FilterChanges {
    FilterChanges {
        mode: Some(value.mode),
        cutoff: Some(value.cutoff),
        resonance: Some(value.resonance),
        envelope_depth: Some(value.envelope_depth),
    }
}
