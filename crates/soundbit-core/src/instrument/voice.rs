//! Sound sources an instrument can use.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{FmVoice, Wavetable};

/// Pulse width of a pulse wave, as found on the NES and Game Boy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DutyCycle {
    /// 12.5 %: thin and nasal.
    Eighth,
    /// 25 %: the classic lead sound.
    Quarter,
    /// 50 %: a hollow square wave.
    #[default]
    Half,
    /// 75 %: sounds like 25 % with inverted phase.
    ThreeQuarters,
}

impl DutyCycle {
    /// The fraction of the cycle the wave is high.
    pub const fn fraction(self) -> f32 {
        match self {
            Self::Eighth => 0.125,
            Self::Quarter => 0.25,
            Self::Half => 0.5,
            Self::ThreeQuarters => 0.75,
        }
    }
}

/// Noise generator mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseMode {
    /// Long pseudo-random sequence: hiss, snares, hi-hats.
    #[default]
    Long,
    /// Short looping sequence: metallic, tonal noise.
    Short,
}

/// The kind of a [`Voice`], without its parameters. Chip profiles constrain these.
///
/// Deliberately exhaustive: adding a kind must be a compile error in every match, so the
/// engine and chip profiles cannot silently ignore it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceKind {
    /// Pulse wave.
    Pulse,
    /// Triangle wave.
    Triangle,
    /// Sawtooth wave.
    Sawtooth,
    /// Sine wave.
    Sine,
    /// Noise.
    Noise,
    /// Custom wavetable.
    Wavetable,
    /// Four-operator FM.
    Fm,
}

impl fmt::Display for VoiceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pulse => "pulse",
            Self::Triangle => "triangle",
            Self::Sawtooth => "sawtooth",
            Self::Sine => "sine",
            Self::Noise => "noise",
            Self::Wavetable => "wavetable",
            Self::Fm => "fm",
        })
    }
}

/// The sound source of an instrument with its parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Voice {
    /// Pulse wave with a selectable duty cycle.
    Pulse {
        /// Pulse width.
        #[serde(default)]
        duty: DutyCycle,
    },
    /// Triangle wave.
    Triangle,
    /// Sawtooth wave.
    Sawtooth,
    /// Sine wave.
    Sine,
    /// Noise generator.
    Noise {
        /// Noise mode.
        #[serde(default)]
        mode: NoiseMode,
    },
    /// Custom single-cycle wavetable.
    Wavetable(Wavetable),
    /// Four-operator FM patch.
    Fm(FmVoice),
}

impl Voice {
    /// The kind of this voice.
    pub const fn kind(&self) -> VoiceKind {
        match self {
            Self::Pulse { .. } => VoiceKind::Pulse,
            Self::Triangle => VoiceKind::Triangle,
            Self::Sawtooth => VoiceKind::Sawtooth,
            Self::Sine => VoiceKind::Sine,
            Self::Noise { .. } => VoiceKind::Noise,
            Self::Wavetable(_) => VoiceKind::Wavetable,
            Self::Fm(_) => VoiceKind::Fm,
        }
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self::Pulse {
            duty: DutyCycle::default(),
        }
    }
}
