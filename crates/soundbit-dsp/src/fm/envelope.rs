//! Per-operator FM envelopes, working in attenuation steps like the YM2612.

use super::tables::{
    ATTENUATION_STEPS, MAX_ATTENUATION, RATE_SECONDS, SUSTAIN_LEVEL_MAX, SUSTAIN_LEVEL_STEP,
};

/// Envelope speeds converted to attenuation steps per sample for one sample rate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct EnvelopeRates {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    sustain_level: f32,
}

impl EnvelopeRates {
    /// Converts YM2612-style rates (`attack`, `decay`, `sustain` 0–31; `release` and
    /// `sustain_level` 0–15).
    pub(super) fn new(
        attack: u8,
        decay: u8,
        sustain: u8,
        release: u8,
        sustain_level: u8,
        sample_rate: f64,
    ) -> Self {
        let sustain_level = sustain_level.min(15);
        Self {
            // The fastest attack rate is instant, as on the YM2612.
            attack: if attack >= 31 {
                INSTANT
            } else {
                steps_per_sample(attack, sample_rate)
            },
            decay: steps_per_sample(decay, sample_rate),
            sustain: steps_per_sample(sustain, sample_rate),
            // Release rates have half the resolution: rate r maps to 2r + 1.
            release: steps_per_sample(release.min(15) * 2 + 1, sample_rate),
            sustain_level: if sustain_level == 15 {
                SUSTAIN_LEVEL_MAX
            } else {
                f32::from(sustain_level) * SUSTAIN_LEVEL_STEP
            },
        }
    }
}

/// A rate that covers the whole attenuation range in one sample.
const INSTANT: f32 = ATTENUATION_STEPS as f32;

/// Attenuation change per sample for a rate; 0 for rate 0, which never moves.
fn steps_per_sample(rate: u8, sample_rate: f64) -> f32 {
    let rate = usize::from(rate.min(31));
    if rate == 0 || sample_rate <= 0.0 {
        0.0
    } else {
        (ATTENUATION_STEPS as f64 / (RATE_SECONDS[rate] * sample_rate)) as f32
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Stage {
    Attack,
    Decay,
    Sustain,
    Release,
    #[default]
    Off,
}

/// An operator envelope: attack, two decay stages and release, in attenuation steps
/// (0 = full level, [`MAX_ATTENUATION`] = silent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct OperatorEnvelope {
    rates: EnvelopeRates,
    stage: Stage,
    attenuation: f32,
}

impl Default for OperatorEnvelope {
    fn default() -> Self {
        Self {
            rates: EnvelopeRates::default(),
            stage: Stage::Off,
            attenuation: MAX_ATTENUATION,
        }
    }
}

impl OperatorEnvelope {
    /// Starts the attack from the current attenuation.
    pub(super) fn key_on(&mut self, rates: EnvelopeRates) {
        self.rates = rates;
        self.stage = Stage::Attack;
        if rates.attack >= MAX_ATTENUATION {
            self.attenuation = 0.0;
            self.stage = Stage::Decay;
        }
    }

    /// Starts the release.
    pub(super) fn key_off(&mut self) {
        if self.stage != Stage::Off {
            self.stage = Stage::Release;
        }
    }

    /// Whether the envelope has fully released.
    pub(super) fn is_off(&self) -> bool {
        self.stage == Stage::Off
    }

    /// Returns the current attenuation and advances by one sample.
    pub(super) fn tick(&mut self) -> f32 {
        let current = self.attenuation;
        let rates = self.rates;
        match self.stage {
            Stage::Off => {}
            Stage::Attack => {
                self.attenuation -= rates.attack;
                if self.attenuation <= 0.0 {
                    self.attenuation = 0.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.attenuation += rates.decay;
                if self.attenuation >= rates.sustain_level {
                    self.attenuation = rates.sustain_level;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {
                self.attenuation = (self.attenuation + rates.sustain).min(MAX_ATTENUATION);
            }
            Stage::Release => {
                self.attenuation += rates.release;
                if self.attenuation >= MAX_ATTENUATION {
                    self.attenuation = MAX_ATTENUATION;
                    self.stage = Stage::Off;
                }
            }
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_attack_then_decays_to_sustain() {
        let rates = EnvelopeRates::new(31, 31, 0, 15, 2, 48_000.0);
        let mut envelope = OperatorEnvelope::default();
        envelope.key_on(rates);
        assert_eq!(envelope.tick(), 0.0);
        for _ in 0..1_000 {
            envelope.tick();
        }
        assert_eq!(envelope.tick(), 64.0);
    }

    #[test]
    fn release_ends_in_silence() {
        let rates = EnvelopeRates::new(31, 0, 0, 15, 0, 48_000.0);
        let mut envelope = OperatorEnvelope::default();
        envelope.key_on(rates);
        envelope.key_off();
        for _ in 0..1_000 {
            envelope.tick();
        }
        assert!(envelope.is_off());
    }

    #[test]
    fn zero_attack_rate_never_rises() {
        let rates = EnvelopeRates::new(0, 31, 0, 15, 0, 48_000.0);
        let mut envelope = OperatorEnvelope::default();
        envelope.key_on(rates);
        for _ in 0..1_000 {
            assert_eq!(envelope.tick(), MAX_ATTENUATION);
        }
    }
}
