//! Tempo and grid settings.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// Tempo in beats per minute.
    Bpm(u16): 20..=400, default 120, "bpm"
}

ranged! {
    /// Grid resolution: how many steps make up one beat.
    StepsPerBeat(u8): 1..=16, default 4, "steps per beat"
}

ranged! {
    /// How many beats make up one bar.
    BeatsPerBar(u8): 1..=16, default 4, "beats per bar"
}

ranged! {
    /// Swing amount in percent. Every second step is delayed by up to half a step; 0 is
    /// straight timing.
    Swing(u8): 0..=100, default 0, "swing"
}

/// Tempo and grid of a song. Every pattern is exactly one bar long.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeSettings {
    /// Tempo.
    pub bpm: Bpm,
    /// Steps per beat.
    pub steps_per_beat: StepsPerBeat,
    /// Beats per bar.
    pub beats_per_bar: BeatsPerBar,
    /// Swing amount.
    pub swing: Swing,
}

impl TimeSettings {
    /// Number of steps in one bar, which is also the length of every pattern.
    pub fn steps_per_bar(&self) -> u16 {
        u16::from(self.steps_per_beat.get()) * u16::from(self.beats_per_bar.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bar_has_sixteen_steps() {
        assert_eq!(TimeSettings::default().steps_per_bar(), 16);
    }

    #[test]
    fn largest_grid_fits_in_u16() {
        let settings = TimeSettings {
            steps_per_beat: StepsPerBeat::MAX,
            beats_per_bar: BeatsPerBar::MAX,
            ..TimeSettings::default()
        };
        assert_eq!(settings.steps_per_bar(), 256);
    }
}
