//! Sawtooth waves.

use super::Oscillator;
use super::blep::poly_blep;
use crate::Phase;

/// A band-limited rising sawtooth wave.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SawOsc {
    phase: Phase,
}

impl Oscillator for SawOsc {
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
        (2.0 * t - 1.0 - poly_blep(t, dt)).clamp(-1.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::test_support::measured_frequency;

    #[test]
    fn plays_requested_frequency() {
        assert_eq!(measured_frequency(&mut SawOsc::default(), 440.0), 440);
    }
}
