//! Partial changes to an instrument's sound shaping. `None` keeps the current value.

use soundbit_core::{
    EnvelopeTime, Filter, FilterCutoff, FilterEnvelopeDepth, FilterMode, FilterResonance,
    PitchEnvelope, PitchSweep, Unison, UnisonDetune, UnisonVoices, Vibrato, VibratoDepth,
    VibratoRate,
};

/// Changes to the pitch envelope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PitchEnvelopeChanges {
    /// New start offset.
    pub semitones: Option<PitchSweep>,
    /// New glide time.
    pub time: Option<EnvelopeTime>,
}

impl PitchEnvelopeChanges {
    /// The pitch envelope with these changes applied.
    pub fn applied_to(self, current: PitchEnvelope) -> PitchEnvelope {
        PitchEnvelope {
            semitones: self.semitones.unwrap_or(current.semitones),
            time: self.time.unwrap_or(current.time),
        }
    }
}

/// Changes to the vibrato.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VibratoChanges {
    /// New depth.
    pub depth: Option<VibratoDepth>,
    /// New rate.
    pub rate: Option<VibratoRate>,
    /// New delay.
    pub delay: Option<EnvelopeTime>,
}

impl VibratoChanges {
    /// The vibrato with these changes applied.
    pub fn applied_to(self, current: Vibrato) -> Vibrato {
        Vibrato {
            depth: self.depth.unwrap_or(current.depth),
            rate: self.rate.unwrap_or(current.rate),
            delay: self.delay.unwrap_or(current.delay),
        }
    }
}

/// Changes to the unison.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnisonChanges {
    /// New number of copies.
    pub voices: Option<UnisonVoices>,
    /// New detune spread.
    pub detune: Option<UnisonDetune>,
}

impl UnisonChanges {
    /// The unison with these changes applied.
    pub fn applied_to(self, current: Unison) -> Unison {
        Unison {
            voices: self.voices.unwrap_or(current.voices),
            detune: self.detune.unwrap_or(current.detune),
        }
    }
}

/// Changes to the filter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilterChanges {
    /// New filter type.
    pub mode: Option<FilterMode>,
    /// New cutoff.
    pub cutoff: Option<FilterCutoff>,
    /// New resonance.
    pub resonance: Option<FilterResonance>,
    /// New envelope depth.
    pub envelope_depth: Option<FilterEnvelopeDepth>,
}

impl FilterChanges {
    /// The filter with these changes applied.
    pub fn applied_to(self, current: Filter) -> Filter {
        Filter {
            mode: self.mode.unwrap_or(current.mode),
            cutoff: self.cutoff.unwrap_or(current.cutoff),
            resonance: self.resonance.unwrap_or(current.resonance),
            envelope_depth: self.envelope_depth.unwrap_or(current.envelope_depth),
        }
    }
}
