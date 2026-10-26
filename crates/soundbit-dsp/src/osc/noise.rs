//! Pseudo-random noise from a linear-feedback shift register.

use super::Oscillator;

/// The LFSR is clocked at this multiple of the note frequency, so higher notes sound brighter.
const CLOCK_PER_HZ: f64 = 8.0;

/// Upper bound on LFSR clocks per output sample, which bounds the work per sample.
const MAX_CLOCKS_PER_SAMPLE: f64 = 16.0;

/// Tap for the long (32767-step) sequence.
const LONG_TAP: u16 = 1;

/// Tap for the short (93-step) metallic sequence.
const SHORT_TAP: u16 = 6;

/// NES-style 15-bit LFSR noise.
///
/// The long mode gives hiss for snares and hi-hats; the short mode loops quickly for metallic,
/// tonal noise. The sequence is fully deterministic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseOsc {
    register: u16,
    tap: u16,
    clocks_per_sample: f64,
    accumulator: f64,
}

impl NoiseOsc {
    /// Creates a noise generator in long or short mode.
    pub fn new(short: bool) -> Self {
        Self {
            register: 1,
            tap: if short { SHORT_TAP } else { LONG_TAP },
            clocks_per_sample: 0.0,
            accumulator: 0.0,
        }
    }

    fn clock(&mut self) {
        let feedback = (self.register ^ (self.register >> self.tap)) & 1;
        self.register = (self.register >> 1) | (feedback << 14);
    }
}

impl Oscillator for NoiseOsc {
    fn set_frequency(&mut self, hz: f64, sample_rate: f64) {
        self.clocks_per_sample = if sample_rate > 0.0 {
            (hz * CLOCK_PER_HZ / sample_rate).clamp(0.0, MAX_CLOCKS_PER_SAMPLE)
        } else {
            0.0
        };
    }

    fn reset(&mut self) {
        self.register = 1;
        self.accumulator = 0.0;
    }

    fn next_sample(&mut self) -> f32 {
        self.accumulator += self.clocks_per_sample;
        while self.accumulator >= 1.0 {
            self.clock();
            self.accumulator -= 1.0;
        }
        if self.register & 1 == 0 { 1.0 } else { -1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn period(short: bool) -> usize {
        let mut osc = NoiseOsc::new(short);
        let start = osc.register;
        (1..=40_000)
            .find(|_| {
                osc.clock();
                osc.register == start
            })
            .expect("LFSR is periodic")
    }

    #[test]
    fn sequences_have_hardware_periods() {
        assert_eq!(period(false), 32_767);
        assert_eq!(period(true), 93);
    }

    #[test]
    fn output_is_balanced_and_bounded() {
        let mut osc = NoiseOsc::new(false);
        osc.set_frequency(4_000.0, 48_000.0);
        let sum: f32 = (0..48_000).map(|_| osc.next_sample()).sum();
        assert!(sum.abs() / 48_000.0 < 0.05, "mean {}", sum / 48_000.0);
    }
}
