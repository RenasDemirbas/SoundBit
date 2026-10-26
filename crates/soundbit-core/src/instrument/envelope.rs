//! Amplitude envelopes.

use serde::{Deserialize, Serialize};

use crate::Level;
use crate::ranged::ranged;

ranged! {
    /// Duration of an envelope stage in milliseconds.
    ///
    /// Sound-design parameters use real time rather than steps, so an instrument sounds the
    /// same at any tempo.
    EnvelopeTime(u16): 0..=10_000, default 0, "envelope time (ms)"
}

/// An attack-decay-sustain-release envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct Envelope {
    /// Time to rise from silence to full level.
    pub attack: EnvelopeTime,
    /// Time to fall from full level to the sustain level.
    pub decay: EnvelopeTime,
    /// Level held while the note is on.
    pub sustain: Level,
    /// Time to fall to silence after the note ends.
    pub release: EnvelopeTime,
}

impl Default for Envelope {
    /// A short, plucky envelope that suits most chiptune leads.
    fn default() -> Self {
        Self {
            attack: EnvelopeTime::saturating(2),
            decay: EnvelopeTime::saturating(120),
            sustain: Level::saturating(70),
            release: EnvelopeTime::saturating(80),
        }
    }
}
