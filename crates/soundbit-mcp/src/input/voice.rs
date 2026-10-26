//! Voices and envelopes.

use schemars::JsonSchema;
use serde::Deserialize;
use soundbit_app::command::EnvelopeChanges;
use soundbit_app::model::{
    DutyCycle, FM_OPERATORS, FmAlgorithm, FmDetune, FmFeedback, FmMultiple, FmOperator, FmRate,
    FmReleaseRate, FmSustainLevel, FmTotalLevel, FmVoice, NoiseMode, Voice, WAVETABLE_LEN,
    WaveLevel, Wavetable,
};

use super::{Choice, optional};

/// The sound source of an instrument.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum VoiceInput {
    /// Pulse (square) wave, the NES and Game Boy lead sound.
    Pulse {
        /// Width of the high part of the wave. Defaults to "half".
        duty: Option<Choice<DutyCycle>>,
    },
    /// NES-style stepped triangle wave, typical for basslines.
    Triangle,
    /// Bright sawtooth wave.
    Sawtooth,
    /// Pure sine wave.
    Sine,
    /// Noise for drums. Higher notes sound brighter.
    Noise {
        /// "long" for hiss, "short" for metallic noise. Defaults to "long".
        mode: Option<Choice<NoiseMode>>,
    },
    /// Custom single-cycle waveform, like the Game Boy wave channel.
    Wavetable {
        /// Exactly 32 levels, each 0-15.
        samples: Vec<u8>,
    },
    /// Four-operator FM, like the Sega Genesis. Unset values use a plain sine patch.
    Fm {
        /// Operator routing 0-7.
        algorithm: Option<u8>,
        /// Operator 1 self-feedback 0-7.
        feedback: Option<u8>,
        /// Up to four operators; missing ones are silent.
        #[serde(default)]
        operators: Vec<FmOperatorInput>,
    },
}

/// One FM operator. Unset values keep the defaults (full level, instant attack, no decay).
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct FmOperatorInput {
    /// Frequency multiplier 0-15 (0 means x0.5).
    multiple: Option<u8>,
    /// Fine detune -3 to 3.
    detune: Option<i8>,
    /// Attenuation 0-127; 0 is loudest. Modulators set brightness with this.
    total_level: Option<u8>,
    /// Attack rate 0-31; 31 is instant.
    attack_rate: Option<u8>,
    /// First decay rate 0-31.
    decay_rate: Option<u8>,
    /// Second decay rate while held, 0-31.
    sustain_rate: Option<u8>,
    /// Level where the first decay ends, 0-15 (0 is loudest).
    sustain_level: Option<u8>,
    /// Release rate 0-15.
    release_rate: Option<u8>,
}

impl VoiceInput {
    pub(crate) fn to_voice(&self) -> Result<Voice, String> {
        Ok(match self {
            Self::Pulse { duty } => Voice::Pulse {
                duty: duty.map(|d| d.0).unwrap_or_default(),
            },
            Self::Triangle => Voice::Triangle,
            Self::Sawtooth => Voice::Sawtooth,
            Self::Sine => Voice::Sine,
            Self::Noise { mode } => Voice::Noise {
                mode: mode.map(|m| m.0).unwrap_or_default(),
            },
            Self::Wavetable { samples } => Voice::Wavetable(wavetable(samples)?),
            Self::Fm {
                algorithm,
                feedback,
                operators,
            } => Voice::Fm(fm_voice(*algorithm, *feedback, operators)?),
        })
    }
}

fn wavetable(samples: &[u8]) -> Result<Wavetable, String> {
    if samples.len() != WAVETABLE_LEN {
        return Err(format!(
            "wavetable needs exactly {WAVETABLE_LEN} samples, got {}",
            samples.len()
        ));
    }
    let mut table = Wavetable::default();
    for (slot, &level) in table.samples.iter_mut().zip(samples) {
        *slot = WaveLevel::new(level).map_err(|error| error.to_string())?;
    }
    Ok(table)
}

fn fm_voice(
    algorithm: Option<u8>,
    feedback: Option<u8>,
    operators: &[FmOperatorInput],
) -> Result<FmVoice, String> {
    if operators.len() > FM_OPERATORS {
        return Err(format!("an FM voice has at most {FM_OPERATORS} operators"));
    }
    let mut voice = FmVoice::default();
    if let Some(algorithm) = optional::<FmAlgorithm, _>(algorithm)? {
        voice.algorithm = algorithm;
    }
    if let Some(feedback) = optional::<FmFeedback, _>(feedback)? {
        voice.feedback = feedback;
    }
    for (index, input) in operators.iter().enumerate() {
        voice.operators[index] = input
            .to_operator()
            .map_err(|error| format!("operators[{index}]: {error}"))?;
    }
    Ok(voice)
}

impl FmOperatorInput {
    fn to_operator(&self) -> Result<FmOperator, String> {
        let base = FmOperator::default();
        Ok(FmOperator {
            multiple: optional::<FmMultiple, _>(self.multiple)?.unwrap_or(base.multiple),
            detune: optional::<FmDetune, _>(self.detune)?.unwrap_or(base.detune),
            total_level: optional::<FmTotalLevel, _>(self.total_level)?.unwrap_or(base.total_level),
            attack_rate: optional::<FmRate, _>(self.attack_rate)?.unwrap_or(base.attack_rate),
            decay_rate: optional::<FmRate, _>(self.decay_rate)?.unwrap_or(base.decay_rate),
            sustain_rate: optional::<FmRate, _>(self.sustain_rate)?.unwrap_or(base.sustain_rate),
            sustain_level: optional::<FmSustainLevel, _>(self.sustain_level)?
                .unwrap_or(base.sustain_level),
            release_rate: optional::<FmReleaseRate, _>(self.release_rate)?
                .unwrap_or(base.release_rate),
        })
    }
}

/// Envelope settings. Unset values keep the current (or default) value.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct EnvelopeInput {
    /// Attack time in milliseconds, 0-10000.
    attack_ms: Option<u16>,
    /// Decay time in milliseconds, 0-10000.
    decay_ms: Option<u16>,
    /// Sustain level in percent, 0-100.
    sustain_percent: Option<u8>,
    /// Release time in milliseconds, 0-10000.
    release_ms: Option<u16>,
}

impl EnvelopeInput {
    pub(crate) fn to_changes(&self) -> Result<EnvelopeChanges, String> {
        Ok(EnvelopeChanges {
            attack: optional(self.attack_ms)?,
            decay: optional(self.decay_ms)?,
            sustain: optional(self.sustain_percent)?,
            release: optional(self.release_ms)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice(json: &str) -> Result<Voice, String> {
        serde_json::from_str::<VoiceInput>(json)
            .map_err(|e| e.to_string())?
            .to_voice()
    }

    #[test]
    fn simple_voices_use_defaults() {
        assert_eq!(
            voice(r#"{"type": "pulse"}"#),
            Ok(Voice::Pulse {
                duty: DutyCycle::Half
            })
        );
        assert_eq!(
            voice(r#"{"type": "noise", "mode": "short"}"#),
            Ok(Voice::Noise {
                mode: NoiseMode::Short
            })
        );
    }

    #[test]
    fn wavetables_need_32_valid_levels() {
        assert!(voice(r#"{"type": "wavetable", "samples": [1, 2]}"#).is_err());
        let levels = serde_json::to_string(&[16_u8; 32]).expect("serializes");
        let error = voice(&format!(r#"{{"type": "wavetable", "samples": {levels}}}"#))
            .expect_err("16 > 15");
        assert!(
            error.contains("wave level must be between 0 and 15"),
            "{error}"
        );
    }

    #[test]
    fn fm_errors_name_the_operator() {
        let error = voice(r#"{"type": "fm", "operators": [{}, {"attack_rate": 40}]}"#)
            .expect_err("rate too high");
        assert_eq!(
            error,
            "operators[1]: FM rate must be between 0 and 31, got 40"
        );
    }
}
