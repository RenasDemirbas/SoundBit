//! Instruments: a voice plus envelope, sound shaping and mix settings.

mod envelope;
mod filter;
mod fm;
mod modulation;
mod unison;
mod voice;
mod wavetable;

use serde::{Deserialize, Serialize};

pub use envelope::{Envelope, EnvelopeTime};
pub use filter::{Filter, FilterCutoff, FilterEnvelopeDepth, FilterMode, FilterResonance};
pub use fm::{
    FM_OPERATORS, FmAlgorithm, FmDetune, FmFeedback, FmMultiple, FmOperator, FmRate, FmReleaseRate,
    FmSustainLevel, FmTotalLevel, FmVoice,
};
pub use modulation::{PitchEnvelope, PitchSweep, Vibrato, VibratoDepth, VibratoRate};
pub use unison::{Unison, UnisonDetune, UnisonVoices};
pub use voice::{DutyCycle, NoiseMode, Voice, VoiceKind};
pub use wavetable::{WAVETABLE_LEN, WaveLevel, Wavetable};

use crate::{Pan, ReverbSend, Volume};

/// A playable sound: what patterns reference to decide how their notes sound.
///
/// Every sound-shaping setting defaults to "off", so an instrument made of just a voice
/// sounds exactly like the bare waveform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    /// Display name.
    pub name: String,
    /// Sound source.
    pub voice: Voice,
    /// Amplitude envelope. FM voices use their per-operator envelopes for their level; the
    /// filter envelope still follows this one.
    #[serde(default)]
    pub envelope: Envelope,
    /// Pitch glide at the start of each note.
    #[serde(default)]
    pub pitch_envelope: PitchEnvelope,
    /// Periodic pitch wobble.
    #[serde(default)]
    pub vibrato: Vibrato,
    /// Detuned copies of the voice.
    #[serde(default)]
    pub unison: Unison,
    /// Resonant filter.
    #[serde(default)]
    pub filter: Filter,
    /// Output volume.
    #[serde(default)]
    pub volume: Volume,
    /// Stereo position.
    #[serde(default)]
    pub pan: Pan,
    /// Share of the sound sent into the song's reverb.
    #[serde(default)]
    pub reverb_send: ReverbSend,
}

impl Instrument {
    /// Creates an instrument with the default envelope, volume and pan, and no sound shaping.
    pub fn new(name: impl Into<String>, voice: Voice) -> Self {
        Self {
            name: name.into(),
            voice,
            envelope: Envelope::default(),
            pitch_envelope: PitchEnvelope::default(),
            vibrato: Vibrato::default(),
            unison: Unison::default(),
            filter: Filter::default(),
            volume: Volume::default(),
            pan: Pan::default(),
            reverb_send: ReverbSend::default(),
        }
    }
}
