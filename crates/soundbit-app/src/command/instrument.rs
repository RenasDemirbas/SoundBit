//! Instrument commands.

use soundbit_core::{
    Envelope, EnvelopeTime, Instrument, InstrumentId, Level, ModelError, Pan, ReverbSend, Song,
    Voice, Volume,
};

use super::{FilterChanges, PitchEnvelopeChanges, UnisonChanges, VibratoChanges};
use crate::CommandError;
use crate::edit::Edit;

/// Changes to an envelope. `None` keeps the current value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnvelopeChanges {
    /// New attack time.
    pub attack: Option<EnvelopeTime>,
    /// New decay time.
    pub decay: Option<EnvelopeTime>,
    /// New sustain level.
    pub sustain: Option<Level>,
    /// New release time.
    pub release: Option<EnvelopeTime>,
}

impl EnvelopeChanges {
    /// The envelope with these changes applied.
    pub fn applied_to(self, envelope: Envelope) -> Envelope {
        Envelope {
            attack: self.attack.unwrap_or(envelope.attack),
            decay: self.decay.unwrap_or(envelope.decay),
            sustain: self.sustain.unwrap_or(envelope.sustain),
            release: self.release.unwrap_or(envelope.release),
        }
    }
}

/// Changes to an instrument. `None` keeps the current value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InstrumentChanges {
    /// New name.
    pub name: Option<String>,
    /// New sound source.
    pub voice: Option<Voice>,
    /// Envelope changes.
    pub envelope: EnvelopeChanges,
    /// Pitch envelope changes.
    pub pitch_envelope: PitchEnvelopeChanges,
    /// Vibrato changes.
    pub vibrato: VibratoChanges,
    /// Unison changes.
    pub unison: UnisonChanges,
    /// Filter changes.
    pub filter: FilterChanges,
    /// New volume.
    pub volume: Option<Volume>,
    /// New pan.
    pub pan: Option<Pan>,
    /// New reverb send.
    pub reverb_send: Option<ReverbSend>,
}

impl InstrumentChanges {
    /// The instrument with these changes applied, e.g. to configure a new instrument.
    pub fn applied_to(self, instrument: &Instrument) -> Instrument {
        Instrument {
            name: self.name.unwrap_or_else(|| instrument.name.clone()),
            voice: self.voice.unwrap_or(instrument.voice),
            envelope: self.envelope.applied_to(instrument.envelope),
            pitch_envelope: self.pitch_envelope.applied_to(instrument.pitch_envelope),
            vibrato: self.vibrato.applied_to(instrument.vibrato),
            unison: self.unison.applied_to(instrument.unison),
            filter: self.filter.applied_to(instrument.filter),
            volume: self.volume.unwrap_or(instrument.volume),
            pan: self.pan.unwrap_or(instrument.pan),
            reverb_send: self.reverb_send.unwrap_or(instrument.reverb_send),
        }
    }
}

/// A change to the song's instruments.
#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentCommand {
    /// Add an instrument.
    Add(Instrument),
    /// Change some properties of an instrument.
    Update(InstrumentId, InstrumentChanges),
    /// Remove an instrument that no pattern uses.
    Remove(InstrumentId),
}

impl InstrumentCommand {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Add(instrument) => format!("Add instrument \"{}\"", instrument.name),
            Self::Update(id, _) => format!("Edit {id}"),
            Self::Remove(id) => format!("Remove {id}"),
        }
    }

    pub(crate) fn into_edit(self, song: &Song) -> Result<Edit, CommandError> {
        Ok(match self {
            Self::Add(instrument) => Edit::AddInstrument(instrument),
            Self::Update(id, changes) => {
                let current = song
                    .instrument(id)
                    .ok_or(ModelError::UnknownInstrument(id))?;
                Edit::ReplaceInstrument(id, changes.applied_to(current))
            }
            Self::Remove(id) => Edit::RemoveInstrument(id),
        })
    }
}
