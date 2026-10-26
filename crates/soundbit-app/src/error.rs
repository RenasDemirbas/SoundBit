//! Application errors.

use std::io;
use std::path::PathBuf;

use soundbit_core::{ModelError, PatternId, Pitch, ProfileViolation};
use soundbit_engine::EngineError;
use soundbit_formats::FormatError;
use thiserror::Error;

/// Live playback failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PlaybackError {
    /// The song could not be compiled for the output's sample rate.
    #[error(transparent)]
    Engine(#[from] EngineError),
    /// The audio thread is not responding.
    #[error(transparent)]
    Player(#[from] soundbit_engine::realtime::PlayerError),
}

/// A command was rejected. The song is unchanged.
///
/// Messages are written for humans and AI agents alike: they say what is wrong and how to fix
/// it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CommandError {
    /// The change would break a model rule.
    #[error(transparent)]
    Model(#[from] ModelError),

    /// A note to remove does not exist.
    #[error("{pattern} has no note {pitch} starting at step {start}")]
    NoteNotFound {
        /// The pattern.
        pattern: PatternId,
        /// Start step of the missing note.
        start: u16,
        /// Pitch of the missing note.
        pitch: Pitch,
    },

    /// The change would add chip-profile violations.
    #[error(
        "rejected because it breaks the song's chip profile: {}. Adjust the change or switch \
         the chip profile to \"free\"",
        join(violations)
    )]
    ProfileViolations {
        /// The violations the change would add.
        violations: Vec<ProfileViolation>,
    },
}

fn join(violations: &[ProfileViolation]) -> String {
    violations
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// A use case failed.
///
/// Messages name the file involved; the cause is available through
/// [`Error::source`](std::error::Error::source). Show the whole chain to users, e.g. with
/// `anyhow`'s `{:#}` formatting.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    /// A file could not be read.
    #[error("cannot read {}", path.display())]
    Read {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },

    /// A file could not be written.
    #[error("cannot write {}", path.display())]
    Write {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },

    /// A file has the wrong format or content.
    #[error("{}", path.display())]
    Format {
        /// The file.
        path: PathBuf,
        /// The underlying error.
        source: FormatError,
    },

    /// A built-in preset file does not parse.
    #[error("built-in preset file {file} is invalid")]
    Presets {
        /// File name.
        file: &'static str,
        /// The problem.
        #[source]
        source: FormatError,
    },

    /// The song could not be rendered.
    #[error(transparent)]
    Engine(#[from] EngineError),
}
