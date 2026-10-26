//! Pulse waves with variable duty cycle.

use super::Oscillator;
use super::blep::poly_blep;
use crate::Phase;
use crate::math::wrap;

/// A band-limited pulse wave.
///
/// Like chip pulse channels it switches between 0 and 1; the output is centred by removing
/// the duty cycle's DC offset, so it spans `[-duty, 1 - duty]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseOsc {
    phase: Phase,
    duty: f64,
}

impl PulseOsc {
    /// Creates a pulse wave whose high part lasts `duty` of the cycle (clamped to 1–99 %).
    pub fn new(duty: f64) -> Self {
        Self {
            phase: Phase::default(),
            duty: duty.clamp(0.01, 0.99),
        }
    }
}

impl Oscillator for PulseOsc {
    fn set_frequency(&mut self, hz: f64, sample_rate: f64) {
        self.phase.set_frequency(hz, sample_rate);
    }

    fn reset(&mut self) {
        self.phase.reset();
    }

    fn set_phase(&mut self, phase: f64) {
        self.phase.set_value(phase);
    }

    fn next_sample(&mut self) -> f32 {
        let dt = self.phase.increment();
        let t = self.phase.tick();
        let naive = if t < self.duty { 1.0 } else { 0.0 };
        // PolyBLEP is defined for steps of height 2; ours are height 1.
        let rise = 0.5 * poly_blep(t, dt);
        let fall = 0.5 * poly_blep(wrap(t + 1.0 - self.duty), dt);
        (naive + rise - fall - self.duty) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::test_support::measured_frequency;

    #[test]
    fn plays_requested_frequency() {
        for duty in [0.125, 0.25, 0.5, 0.75] {
            assert_eq!(
                measured_frequency(&mut PulseOsc::new(duty), 440.0),
                440,
                "duty {duty}"
            );
        }
    }

    #[test]
    fn has_no_dc_offset() {
        let mut osc = PulseOsc::new(0.125);
        osc.set_frequency(480.0, 48_000.0);
        let sum: f64 = (0..48_000).map(|_| f64::from(osc.next_sample())).sum();
        assert!(sum.abs() / 48_000.0 < 1e-3, "mean {}", sum / 48_000.0);
    }
}
