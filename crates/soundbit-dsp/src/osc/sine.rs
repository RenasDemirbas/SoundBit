//! Sine waves.

use super::Oscillator;
use crate::Phase;
use crate::math::sine;

/// A pure sine wave.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SineOsc {
    phase: Phase,
}

impl Oscillator for SineOsc {
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
        sine(self.phase.tick()) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::test_support::measured_frequency;

    #[test]
    fn plays_requested_frequency() {
        assert_eq!(measured_frequency(&mut SineOsc::default(), 440.0), 440);
    }
}
