//! Stepped triangle waves.

use super::Oscillator;
use crate::Phase;

/// Steps per triangle cycle, as on the NES triangle channel.
const STEPS: f64 = 32.0;

/// A 4-bit stepped triangle wave in the style of the NES triangle channel: 16 levels down,
/// 16 levels up. The steps give it its characteristic buzzy, "8-bit" bass.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TriangleOsc {
    phase: Phase,
}

impl Oscillator for TriangleOsc {
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
        let step = ((self.phase.tick() * STEPS) as u8).min(31);
        let level = if step < 16 { 15 - step } else { step - 16 };
        f32::from(level) / 7.5 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::test_support::measured_frequency;

    #[test]
    fn plays_requested_frequency() {
        assert_eq!(measured_frequency(&mut TriangleOsc::default(), 440.0), 440);
    }

    #[test]
    fn uses_sixteen_levels() {
        let mut osc = TriangleOsc::default();
        osc.set_frequency(1_500.0, 48_000.0);
        let mut levels: std::vec::Vec<i32> = (0..32)
            .map(|_| (osc.next_sample() * 7.5).round() as i32)
            .collect();
        levels.sort_unstable();
        levels.dedup();
        assert_eq!(levels.len(), 16);
    }
}
