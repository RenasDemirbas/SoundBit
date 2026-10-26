//! Offline rendering of a whole song.

use soundbit_core::Song;

use crate::engine::{CHANNELS, Engine};
use crate::error::EngineError;
use crate::meter::TrackPower;
use crate::schedule::Schedule;

/// Frames rendered per engine call; matches a typical real-time buffer size.
const BLOCK_FRAMES: usize = 512;

/// Settings for [`render`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    /// Output sample rate in hertz.
    pub sample_rate: u32,
    /// Longest time notes may keep releasing after the arrangement ends.
    pub max_tail_seconds: u32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            max_tail_seconds: 5,
        }
    }
}

/// Rendered audio: interleaved stereo samples in `[-1, 1]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    /// Sample rate in hertz.
    pub sample_rate: u32,
    /// Interleaved samples, [`CHANNELS`] per frame.
    pub samples: Vec<f32>,
    /// Level of each track's contribution, in track order.
    pub tracks: Vec<TrackPower>,
}

impl Rendered {
    /// Number of stereo frames.
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(CHANNELS)
    }

    /// Duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.frames() as f64 / f64::from(self.sample_rate)
    }

    /// Mean of the squared samples; its square root is the RMS level.
    pub fn mean_square(&self) -> f64 {
        let sum: f64 = self
            .samples
            .iter()
            .map(|&sample| f64::from(sample) * f64::from(sample))
            .sum();
        sum / self.samples.len().max(1) as f64
    }

    /// Largest absolute sample value.
    pub fn peak(&self) -> f32 {
        self.samples
            .iter()
            .fold(0.0, |peak, sample| peak.max(sample.abs()))
    }
}

/// Renders `song` from start to finish, including release and reverb tails.
///
/// The output is bit-exact for the same song and options on every platform.
///
/// # Errors
///
/// Returns [`EngineError::InvalidSampleRate`] for unsupported sample rates.
pub fn render(song: &Song, options: &RenderOptions) -> Result<Rendered, EngineError> {
    let schedule = Schedule::compile(song, options.sample_rate)?;
    let limit = schedule.end_sample()
        + u64::from(options.max_tail_seconds) * u64::from(options.sample_rate);
    let mut engine = Engine::new(schedule);
    engine.enable_metering();

    let mut samples = Vec::new();
    let mut block = [0.0; BLOCK_FRAMES * CHANNELS as usize];
    while !engine.is_finished() && engine.position() < limit {
        let remaining = usize::try_from(limit - engine.position()).unwrap_or(usize::MAX);
        let frames = remaining.min(BLOCK_FRAMES);
        let out = &mut block[..frames * usize::from(CHANNELS)];
        engine.process(out);
        samples.extend_from_slice(out);
    }
    Ok(Rendered {
        sample_rate: options.sample_rate,
        samples,
        tracks: engine.track_power(),
    })
}
