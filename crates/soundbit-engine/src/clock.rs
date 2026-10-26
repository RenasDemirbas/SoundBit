//! Conversion from musical steps to sample positions.

use soundbit_core::TimeSettings;

/// Maps step positions to sample positions with exact integer arithmetic, so timing never
/// drifts and is identical on every platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StepClock {
    /// `sample_rate * 60`: samples per minute.
    samples_per_minute: u64,
    /// `bpm * steps_per_beat`: steps per minute.
    steps_per_minute: u64,
    /// Delay applied to every odd step.
    swing_delay: u64,
}

impl StepClock {
    pub(crate) fn new(timing: TimeSettings, sample_rate: u32) -> Self {
        let samples_per_minute = u64::from(sample_rate) * 60;
        let steps_per_minute = u64::from(timing.bpm.get()) * u64::from(timing.steps_per_beat.get());
        // Full swing (100 %) delays odd steps by half a step.
        let swing_delay =
            samples_per_minute * u64::from(timing.swing.get()) / (steps_per_minute * 200);
        Self {
            samples_per_minute,
            steps_per_minute,
            swing_delay,
        }
    }

    /// The step playing at `sample`, ignoring swing. Used to show the playback position.
    pub(crate) fn step_at(&self, sample: u64) -> u64 {
        sample * self.steps_per_minute / self.samples_per_minute
    }

    /// The sample at which `step` begins.
    pub(crate) fn sample_at(&self, step: u64) -> u64 {
        let straight = step * self.samples_per_minute / self.steps_per_minute;
        if step % 2 == 1 {
            straight + self.swing_delay
        } else {
            straight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundbit_core::{Bpm, Swing};

    fn timing(bpm: u16, swing: u8) -> TimeSettings {
        TimeSettings {
            bpm: Bpm::new(bpm).expect("valid bpm"),
            swing: Swing::new(swing).expect("valid swing"),
            ..TimeSettings::default()
        }
    }

    #[test]
    fn straight_steps_are_evenly_spaced() {
        // 120 bpm, 4 steps per beat: 8 steps per second, 6000 samples per step at 48 kHz.
        let clock = StepClock::new(timing(120, 0), 48_000);
        let starts: Vec<u64> = (0..4).map(|step| clock.sample_at(step)).collect();
        assert_eq!(starts, [0, 6_000, 12_000, 18_000]);
    }

    #[test]
    fn fractional_steps_do_not_drift() {
        // 140 bpm: 5142.857... samples per step. Rounding each step and summing would drift to
        // 35994 after seven steps; computing from the start gives exactly 36000.
        let clock = StepClock::new(timing(140, 0), 48_000);
        assert_eq!(clock.sample_at(1), 5_142);
        assert_eq!(clock.sample_at(7), 36_000);
    }

    #[test]
    fn swing_delays_odd_steps() {
        let clock = StepClock::new(timing(120, 50), 48_000);
        assert_eq!(clock.sample_at(1), 6_000 + 1_500);
        assert_eq!(clock.sample_at(2), 12_000);
    }
}
