//! Engine errors.

use thiserror::Error;

/// Lowest supported sample rate in hertz.
pub const MIN_SAMPLE_RATE: u32 = 8_000;

/// Highest supported sample rate in hertz.
pub const MAX_SAMPLE_RATE: u32 = 192_000;

/// A song could not be prepared for playback.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum EngineError {
    /// The requested sample rate is not supported.
    #[error("sample rate must be between {min} and {max} Hz, got {got}")]
    InvalidSampleRate {
        /// Requested sample rate.
        got: u32,
        /// Lowest supported rate.
        min: u32,
        /// Highest supported rate.
        max: u32,
    },
}

/// Checks that `sample_rate` is supported.
pub(crate) fn check_sample_rate(sample_rate: u32) -> Result<(), EngineError> {
    if (MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate) {
        Ok(())
    } else {
        Err(EngineError::InvalidSampleRate {
            got: sample_rate,
            min: MIN_SAMPLE_RATE,
            max: MAX_SAMPLE_RATE,
        })
    }
}
