//! Audio device output.
//!
//! [`AudioOutput`] opens the system's default output device and calls a render callback for
//! interleaved stereo `f32` frames. It knows nothing about songs: the composition root passes a
//! closure that drives the engine's real-time player.
//!
//! Supported on Linux (ALSA, and `PipeWire`/`PulseAudio` through ALSA), Windows (WASAPI) and
//! macOS (`CoreAudio`) via `cpal`.

#![forbid(unsafe_code)]

mod error;
mod frames;
mod output;

pub use error::AudioError;
pub use output::{AudioOutput, OutputInfo};
