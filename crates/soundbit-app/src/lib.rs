//! Application layer.
//!
//! The only place where song state is mutated. Adapters (GUI, MCP, CLI) express changes as
//! [`Command`]s and run them through an [`Editor`], which applies them atomically, enforces the
//! chip-profile policy and keeps the undo/redo history. A [`Session`] adds the project file.
//!
//! Adapters depend on this crate only; it re-exports the domain and engine types they need.

#![forbid(unsafe_code)]

pub mod command;
mod edit;
mod editor;
mod error;
mod history;
pub mod playback;
mod presets;
mod session;

pub use command::Command;
pub use edit::Created;
pub use editor::{Editor, Origin, Outcome, ProfilePolicy};
pub use error::{AppError, CommandError, PlaybackError};
pub use playback::Playback;
pub use presets::PresetLibrary;
pub use session::{ExportReport, Session, SharedSession, TrackLevel};
pub use soundbit_core as model;
pub use soundbit_engine::RenderOptions;
pub use soundbit_formats::Preset;
