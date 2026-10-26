//! 4-bit wavetables in the style of the Game Boy wave channel.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// One 4-bit wavetable sample.
    WaveLevel(u8): 0..=15, default 0, "wave level"
}

/// Number of samples in a wavetable.
pub const WAVETABLE_LEN: usize = 32;

/// A single-cycle waveform of 32 four-bit samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct Wavetable {
    /// Samples of one waveform cycle.
    pub samples: [WaveLevel; WAVETABLE_LEN],
}

impl Wavetable {
    /// A triangle wave: rises over the first half of the cycle, falls over the second.
    pub const fn triangle() -> Self {
        let mut samples = [WaveLevel::MIN; WAVETABLE_LEN];
        let mut index: u8 = 0;
        while (index as usize) < WAVETABLE_LEN {
            let level = if index < 16 { index } else { 31 - index };
            samples[index as usize] = WaveLevel::saturating(level);
            index += 1;
        }
        Self { samples }
    }
}

impl Default for Wavetable {
    fn default() -> Self {
        Self::triangle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_spans_full_range() {
        let table = Wavetable::triangle();
        assert_eq!(table.samples[0], WaveLevel::MIN);
        assert_eq!(table.samples[15], WaveLevel::MAX);
        assert_eq!(table.samples[16], WaveLevel::MAX);
        assert_eq!(table.samples[31], WaveLevel::MIN);
    }
}
