//! FM operators: a sine oscillator with its own envelope.

use super::envelope::{EnvelopeRates, OperatorEnvelope};
use super::tables::{AMPLITUDE, DETUNE_STEP, MAX_ATTENUATION, TOTAL_LEVEL_STEP};
use crate::Phase;
use crate::math::sine;

/// Raw operator parameters with YM2612 names and ranges, as stored in a song.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct YmOperator {
    /// Frequency multiplier (0 = ×0.5, 1–15 = ×1–×15).
    pub multiple: u8,
    /// Fine detune (-3–3).
    pub detune: i8,
    /// Attenuation in 0.75 dB steps (0–127).
    pub total_level: u8,
    /// Attack rate (0–31).
    pub attack_rate: u8,
    /// First decay rate (0–31).
    pub decay_rate: u8,
    /// Second decay rate while held (0–31).
    pub sustain_rate: u8,
    /// Level where the first decay ends, in 3 dB steps (0–15).
    pub sustain_level: u8,
    /// Release rate (0–15).
    pub release_rate: u8,
}

/// Operator parameters converted for one sample rate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OperatorParams {
    ratio: f64,
    total_level: f32,
    rates: EnvelopeRates,
}

impl OperatorParams {
    /// Converts raw parameters. Out-of-range values are clamped.
    pub fn from_ym(op: &YmOperator, sample_rate: f64) -> Self {
        let multiple = match op.multiple.min(15) {
            0 => 0.5,
            n => f64::from(n),
        };
        let detune = f64::from(op.detune.clamp(-3, 3));
        Self {
            ratio: multiple * (1.0 + detune * DETUNE_STEP),
            total_level: f32::from(op.total_level.min(127)) * TOTAL_LEVEL_STEP,
            rates: EnvelopeRates::new(
                op.attack_rate,
                op.decay_rate,
                op.sustain_rate,
                op.release_rate,
                op.sustain_level,
                sample_rate,
            ),
        }
    }
}

/// A playing operator.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Operator {
    params: OperatorParams,
    phase: Phase,
    envelope: OperatorEnvelope,
}

impl Operator {
    pub(super) fn key_on(&mut self, params: OperatorParams, base_hz: f64, sample_rate: f64) {
        self.params = params;
        self.phase
            .set_frequency(base_hz * params.ratio, sample_rate);
        self.phase.reset();
        self.envelope.key_on(params.rates);
    }

    pub(super) fn set_frequency(&mut self, base_hz: f64, sample_rate: f64) {
        self.phase
            .set_frequency(base_hz * self.params.ratio, sample_rate);
    }

    pub(super) fn key_off(&mut self) {
        self.envelope.key_off();
    }

    pub(super) fn is_off(&self) -> bool {
        self.envelope.is_off()
    }

    /// Produces the next output, with `modulation` added to the phase (in cycles).
    pub(super) fn output(&mut self, modulation: f64) -> f64 {
        let attenuation = (self.envelope.tick() + self.params.total_level).min(MAX_ATTENUATION);
        let amplitude = AMPLITUDE[attenuation as usize];
        sine(self.phase.tick() + modulation) * f64::from(amplitude)
    }
}
