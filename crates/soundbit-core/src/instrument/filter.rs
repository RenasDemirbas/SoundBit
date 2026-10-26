//! A resonant filter per instrument.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// Filter cutoff frequency in hertz.
    FilterCutoff(u16): 20..=20_000, default 2_000, "filter cutoff (Hz)"
}

ranged! {
    /// Emphasis around the cutoff in percent. High values whistle.
    FilterResonance(u8): 0..=100, default 0, "filter resonance"
}

ranged! {
    /// How far the instrument's amplitude envelope moves the cutoff, in semitones at full
    /// level. Positive values open the filter as the note starts, negative ones close it.
    FilterEnvelopeDepth(i8): -72..=72, default 0, "filter envelope depth (semitones)"
}

/// Which frequencies a filter lets through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterMode {
    /// No filtering.
    #[default]
    Off,
    /// Keeps frequencies below the cutoff: softer, darker, warmer.
    LowPass,
    /// Keeps frequencies above the cutoff: thinner, brighter.
    HighPass,
    /// Keeps frequencies around the cutoff: nasal, telephone-like.
    BandPass,
}

/// A resonant state-variable filter applied to every note of an instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Filter {
    /// Filter type; [`FilterMode::Off`] bypasses the filter.
    pub mode: FilterMode,
    /// Cutoff frequency.
    pub cutoff: FilterCutoff,
    /// Resonance.
    pub resonance: FilterResonance,
    /// Cutoff movement following the amplitude envelope.
    pub envelope_depth: FilterEnvelopeDepth,
}

impl Filter {
    /// Whether the filter is bypassed.
    pub fn is_off(&self) -> bool {
        self.mode == FilterMode::Off
    }
}
