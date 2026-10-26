//! Sequencer, voice allocation, mixer and offline renderer.
//!
//! A [`Song`](soundbit_core::Song) is first compiled into a [`Schedule`]: instruments become
//! DSP patches and notes become sample-accurate events. This happens off the audio thread.
//! The [`Engine`] then plays a schedule; [`Engine::process`] is real-time safe. [`render`] uses
//! the same engine to render a whole song offline, bit-exact for identical input, and
//! [`realtime`] drives it from an audio callback for live playback.

#![forbid(unsafe_code)]
#![allow(
    clippy::cast_precision_loss,
    reason = "converting sample counts and parameters to floats is inherent to audio code"
)]

mod clock;
mod engine;
mod error;
mod meter;
mod patch;
pub mod realtime;
mod render;
mod schedule;
mod shaping;
mod source;
mod track;
mod voice;

pub use engine::{CHANNELS, Engine};
pub use error::{EngineError, MAX_SAMPLE_RATE, MIN_SAMPLE_RATE};
pub use meter::TrackPower;
pub use render::{RenderOptions, Rendered, render};
pub use schedule::{PositionMap, Schedule};
