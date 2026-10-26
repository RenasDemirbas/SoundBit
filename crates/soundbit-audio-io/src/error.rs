//! Audio output errors.

use thiserror::Error;

/// The audio device could not be opened.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AudioError {
    /// The system reports no default output device.
    #[error("no audio output device found")]
    NoDevice,

    /// The device rejected the request.
    #[error("the audio device failed")]
    Device(#[from] cpal::Error),

    /// The device uses a sample format SoundBit does not convert to.
    #[error("the audio device uses an unsupported sample format ({0})")]
    UnsupportedFormat(String),

    /// The audio thread could not be started.
    #[error("cannot start the audio thread")]
    Thread(#[from] std::io::Error),

    /// The audio thread stopped before reporting whether the device opened.
    #[error("the audio thread stopped unexpectedly")]
    ThreadStopped,
}
