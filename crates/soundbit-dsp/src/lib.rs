//! Real-time safe DSP building blocks.
//!
//! Oscillators, envelopes, FM synthesis, pitch modulation, filters and reverb, operating one
//! sample at a time. Everything here may be called from the audio callback and follows the
//! real-time rules in `CONTRIBUTING.md`. The crate is `no_std`, so heap allocation, I/O,
//! locking and printing are impossible by construction.
//!
//! Output is bit-identical on every platform: only exactly-rounded IEEE-754 operations are used,
//! and all lookup tables are computed at compile time (see [`math`]).

#![no_std]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "DSP code converts between sample counts, table indices and floats by design"
)]

#[cfg(test)]
extern crate std;

pub mod envelope;
pub mod filter;
pub mod fm;
pub mod math;
pub mod modulation;
pub mod osc;
mod phase;
pub mod reverb;

pub use envelope::{Adsr, AdsrParams};
pub use filter::{FilterResponse, Svf};
pub use fm::{FmParams, FmSynth, OperatorParams, YmOperator};
pub use modulation::{PitchMod, PitchModParams};
pub use osc::{
    NoiseOsc, Oscillator, PulseOsc, SawOsc, SineOsc, TriangleOsc, WAVETABLE_LEN, WavetableOsc,
};
pub use phase::Phase;
pub use reverb::{Reverb, ReverbParams};
