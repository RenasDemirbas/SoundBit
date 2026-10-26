//! Four-operator FM voices.
//!
//! Parameter names and ranges follow the Yamaha YM2612 (Sega Genesis / Mega Drive), so
//! patches from that ecosystem translate directly.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// Operator routing algorithm. 0 is a full serial stack; 7 is four parallel carriers.
    FmAlgorithm(u8): 0..=7, default 7, "FM algorithm"
}

ranged! {
    /// Self-feedback amount of operator 1.
    FmFeedback(u8): 0..=7, default 0, "FM feedback"
}

ranged! {
    /// Frequency multiplier. 0 means x0.5; 1 to 15 mean x1 to x15.
    FmMultiple(u8): 0..=15, default 1, "FM multiple"
}

ranged! {
    /// Fine detune.
    FmDetune(i8): -3..=3, default 0, "FM detune"
}

ranged! {
    /// Attenuation: 0 is loudest, 127 is silent.
    FmTotalLevel(u8): 0..=127, default 0, "FM total level"
}

ranged! {
    /// Attack, decay or sustain rate: higher is faster.
    FmRate(u8): 0..=31, default 31, "FM rate"
}

ranged! {
    /// Release rate: higher is faster.
    FmReleaseRate(u8): 0..=15, default 7, "FM release rate"
}

ranged! {
    /// Level where decay hands over to sustain: 0 is loudest, 15 is silent.
    FmSustainLevel(u8): 0..=15, default 0, "FM sustain level"
}

/// Number of operators in an FM voice.
pub const FM_OPERATORS: usize = 4;

/// One FM operator: an oscillator with its own envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct FmOperator {
    /// Frequency multiplier.
    pub multiple: FmMultiple,
    /// Fine detune.
    pub detune: FmDetune,
    /// Attenuation.
    pub total_level: FmTotalLevel,
    /// Attack rate.
    pub attack_rate: FmRate,
    /// First decay rate.
    pub decay_rate: FmRate,
    /// Second decay rate while the note is held.
    pub sustain_rate: FmRate,
    /// Level where the first decay ends.
    pub sustain_level: FmSustainLevel,
    /// Release rate.
    pub release_rate: FmReleaseRate,
}

impl FmOperator {
    /// A fully attenuated operator that contributes nothing.
    pub const SILENT: Self = Self {
        multiple: FmMultiple::saturating(1),
        detune: FmDetune::saturating(0),
        total_level: FmTotalLevel::MAX,
        attack_rate: FmRate::MAX,
        decay_rate: FmRate::saturating(0),
        sustain_rate: FmRate::saturating(0),
        sustain_level: FmSustainLevel::MIN,
        release_rate: FmReleaseRate::saturating(7),
    };
}

impl Default for FmOperator {
    /// A full-level operator with an instant attack and no decay.
    fn default() -> Self {
        Self {
            total_level: FmTotalLevel::MIN,
            ..Self::SILENT
        }
    }
}

/// A complete four-operator FM patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct FmVoice {
    /// Operator routing.
    pub algorithm: FmAlgorithm,
    /// Operator 1 self-feedback.
    pub feedback: FmFeedback,
    /// Operators 1 to 4.
    pub operators: [FmOperator; FM_OPERATORS],
}

impl Default for FmVoice {
    /// A plain sine: one audible carrier, the other operators silent.
    fn default() -> Self {
        Self {
            algorithm: FmAlgorithm::MAX,
            feedback: FmFeedback::default(),
            operators: [
                FmOperator::default(),
                FmOperator::SILENT,
                FmOperator::SILENT,
                FmOperator::SILENT,
            ],
        }
    }
}
