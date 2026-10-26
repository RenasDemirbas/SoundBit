//! Volume, panning and generic percentage levels.

use crate::ranged::ranged;

ranged! {
    /// Output volume of an instrument or track, in percent.
    Volume(u8): 0..=100, default 80, "volume"
}

ranged! {
    /// Stereo position: -100 is hard left, 0 is centre, 100 is hard right.
    Pan(i8): -100..=100, default 0, "pan"
}

ranged! {
    /// A generic level in percent, e.g. an envelope's sustain level.
    Level(u8): 0..=100, default 100, "level"
}
