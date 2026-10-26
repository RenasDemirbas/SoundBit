//! Format errors.

use thiserror::Error;

/// A file could not be read or written.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FormatError {
    /// The JSON is valid but is not a SoundBit project.
    #[error("not a SoundBit project: expected \"format\": \"soundbit-project\"")]
    NotAProject,

    /// The JSON is valid but is not a SoundBit preset file.
    #[error("not a SoundBit preset file: expected \"format\": \"soundbit-presets\"")]
    NotAPresetFile,

    /// The file was written by a newer (or invalid) format version.
    #[error(
        "format version {found} is not supported; this build reads version {supported} and \
         older (try a newer SoundBit)"
    )]
    UnsupportedVersion {
        /// Version found in the file.
        found: u32,
        /// Newest version this build reads.
        supported: u32,
    },

    /// The file is not valid JSON or does not describe a valid song.
    #[error("invalid project")]
    Json(#[from] serde_json::Error),

    /// The audio does not fit in a WAV file (4 GiB limit).
    #[error("audio is too long for a WAV file")]
    AudioTooLong,

    /// Writing failed.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
