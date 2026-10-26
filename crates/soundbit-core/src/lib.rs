//! Pure domain model for SoundBit songs.
//!
//! Songs, instruments, patterns, tracks, the arrangement and chip profiles live here as plain
//! data plus validation. This crate has no I/O, no audio and no async, and depends on no other
//! workspace crate.
//!
//! [`Song`] is the aggregate root: all changes go through its methods, which keep the song
//! consistent (every reference resolves, every note fits its bar, every value is in range).
//! Musical time is measured in steps; seconds and samples only exist in the engine.

#![forbid(unsafe_code)]

mod arrangement;
mod chip;
mod collection;
mod error;
mod id;
mod instrument;
mod levels;
pub mod limits;
mod note;
mod pattern;
mod pitch;
mod ranged;
mod reverb;
mod scale;
mod song;
mod timing;

pub use arrangement::{LoopRange, Track, TrackSettings};
pub use chip::{Channel, ChipProfile, ProfileViolation};
pub use collection::Removed;
pub use error::ModelError;
pub use id::{InstrumentId, PatternId, TrackId};
pub use instrument::{
    DutyCycle, Envelope, EnvelopeTime, FM_OPERATORS, Filter, FilterCutoff, FilterEnvelopeDepth,
    FilterMode, FilterResonance, FmAlgorithm, FmDetune, FmFeedback, FmMultiple, FmOperator, FmRate,
    FmReleaseRate, FmSustainLevel, FmTotalLevel, FmVoice, Instrument, NoiseMode, PitchEnvelope,
    PitchSweep, Unison, UnisonDetune, UnisonVoices, Vibrato, VibratoDepth, VibratoRate, Voice,
    VoiceKind, WAVETABLE_LEN, WaveLevel, Wavetable,
};
pub use levels::{Level, Pan, Volume};
pub use note::{Note, Velocity};
pub use pattern::Pattern;
pub use pitch::{Pitch, PitchClass};
pub use reverb::{Reverb, ReverbDamping, ReverbSend, RoomSize};
pub use scale::{Scale, ScaleKind};
pub use song::{Song, SongMeta};
pub use timing::{BeatsPerBar, Bpm, StepsPerBeat, Swing, TimeSettings};
