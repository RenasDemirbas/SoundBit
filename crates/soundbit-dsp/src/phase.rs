//! Phase accumulation.

/// Position within a waveform cycle, in `[0, 1)`, advanced once per sample.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Phase {
    value: f64,
    increment: f64,
}

/// Highest supported frequency as a fraction of the sample rate (Nyquist).
const MAX_INCREMENT: f64 = 0.5;

impl Phase {
    /// Sets the frequency. Frequencies above Nyquist are clamped.
    pub fn set_frequency(&mut self, hz: f64, sample_rate: f64) {
        self.increment = if sample_rate > 0.0 {
            (hz / sample_rate).clamp(0.0, MAX_INCREMENT)
        } else {
            0.0
        };
    }

    /// Current phase.
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Phase advance per sample.
    pub const fn increment(&self) -> f64 {
        self.increment
    }

    /// Jumps to a position in the cycle; wrapped into `[0, 1)`.
    pub fn set_value(&mut self, value: f64) {
        self.value = crate::math::wrap(value);
    }

    /// Restarts the cycle.
    pub fn reset(&mut self) {
        self.value = 0.0;
    }

    /// Returns the current phase and advances by one sample.
    pub fn tick(&mut self) -> f64 {
        let current = self.value;
        self.value += self.increment;
        if self.value >= 1.0 {
            self.value -= 1.0;
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_once_per_cycle() {
        let mut phase = Phase::default();
        phase.set_frequency(1_000.0, 8_000.0);
        let values: std::vec::Vec<f64> = (0..9).map(|_| phase.tick()).collect();
        assert_eq!(
            values,
            [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 0.0]
        );
    }

    #[test]
    fn clamps_above_nyquist() {
        let mut phase = Phase::default();
        phase.set_frequency(30_000.0, 48_000.0);
        assert_eq!(phase.increment(), 0.5);
    }
}
