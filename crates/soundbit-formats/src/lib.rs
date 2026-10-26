//! Persistence and interchange formats.
//!
//! The versioned SoundBit project file (`.sbproj`), instrument preset files and WAV export. Bosca Ceoil (`.ceol`)
//! import and MIDI export will live here as well.

#![forbid(unsafe_code)]

mod error;
mod migrations;
mod preset;
mod project;
mod wav;

pub use error::FormatError;
pub use preset::{PRESET_FORMAT_VERSION, Preset, parse_presets};
pub use project::{FORMAT_VERSION, PROJECT_EXTENSION, from_project_json, to_project_json};
pub use wav::write_wav;
