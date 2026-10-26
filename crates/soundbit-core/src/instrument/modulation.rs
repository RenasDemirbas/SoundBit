//! Pitch modulation: a pitch sweep at the start of each note, and vibrato.

use serde::{Deserialize, Serialize};

use super::EnvelopeTime;
use crate::ranged::ranged;

ranged! {
    /// Pitch offset at the start of a note in semitones. The pitch glides from here to the
    /// note's own pitch. Positive values fall into the note (kicks, toms), negative ones rise.
    PitchSweep(i8): -48..=48, default 0, "pitch sweep (semitones)"
}

ranged! {
    /// Vibrato depth in cents (hundredths of a semitone) either side of the note.
    VibratoDepth(u8): 0..=100, default 0, "vibrato depth (cents)"
}

ranged! {
    /// Vibrato speed in tenths of a hertz: 55 means 5.5 wobbles per second.
    VibratoRate(u8): 1..=200, default 55, "vibrato rate (0.1 Hz)"
}

/// A glide from an offset pitch to the note's pitch over the first moments of a note.
///
/// This is what makes a sine sound like a kick drum or a timpani hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PitchEnvelope {
    /// Where the pitch starts, relative to the note.
    pub semitones: PitchSweep,
    /// How long the glide to the note's pitch takes.
    pub time: EnvelopeTime,
}

impl PitchEnvelope {
    /// Whether the envelope leaves the pitch unchanged.
    pub fn is_off(&self) -> bool {
        self.semitones.get() == 0 || self.time.get() == 0
    }
}

/// A periodic wobble of the pitch that brings held notes to life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Vibrato {
    /// How far the pitch moves.
    pub depth: VibratoDepth,
    /// How fast it moves.
    pub rate: VibratoRate,
    /// How long a note plays before the vibrato starts, as string and wind players do.
    pub delay: EnvelopeTime,
}

impl Vibrato {
    /// Whether the vibrato leaves the pitch unchanged.
    pub fn is_off(&self) -> bool {
        self.depth.get() == 0
    }
}
