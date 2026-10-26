//! Unison: several slightly detuned copies of a voice playing together.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// Number of copies of the voice that sound for every note. 1 turns unison off.
    UnisonVoices(u8): 1..=4, default 1, "unison voices"
}

ranged! {
    /// How far the outermost copies are detuned, in cents either side of the note.
    UnisonDetune(u8): 0..=50, default 12, "unison detune (cents)"
}

/// Detuned copies of the voice, which turn a thin waveform into an ensemble (strings, choirs,
/// supersaw leads).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Unison {
    /// Number of copies.
    pub voices: UnisonVoices,
    /// Spread of the copies.
    pub detune: UnisonDetune,
}

impl Unison {
    /// Whether only a single copy plays.
    pub fn is_off(&self) -> bool {
        self.voices.get() == 1
    }
}
