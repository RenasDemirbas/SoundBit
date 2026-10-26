//! Single-cycle wavetable playback.

use super::Oscillator;
use crate::Phase;

/// Number of samples in a wavetable.
pub const WAVETABLE_LEN: usize = 32;

/// Plays a 32-sample waveform without interpolation, like the Game Boy wave channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WavetableOsc {
    phase: Phase,
    table: [f32; WAVETABLE_LEN],
}

impl WavetableOsc {
    /// Creates an oscillator for a table of samples in `[-1, 1]`.
    pub fn new(table: [f32; WAVETABLE_LEN]) -> Self {
        Self {
            phase: Phase::default(),
            table: table.map(|s| s.clamp(-1.0, 1.0)),
        }
    }
}

impl Oscillator for WavetableOsc {
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
        let index = (self.phase.tick() * WAVETABLE_LEN as f64) as usize;
        self.table[index.min(WAVETABLE_LEN - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osc::test_support::measured_frequency;

    #[test]
    fn plays_table_at_requested_frequency() {
        let square = core::array::from_fn(|i| if i < 16 { 1.0 } else { -1.0 });
        let mut osc = WavetableOsc::new(square);
        assert_eq!(measured_frequency(&mut osc, 300.0), 300);
    }
}
