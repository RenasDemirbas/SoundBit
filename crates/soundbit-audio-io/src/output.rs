//! The default output device, driven by a render callback.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};

use crate::AudioError;
use crate::frames::write_frames;

/// Stereo frames rendered per callback chunk. Larger device buffers are rendered in chunks.
const CHUNK_FRAMES: usize = 1024;

/// What was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputInfo {
    /// Device name.
    pub device: String,
    /// Sample rate the render callback must produce.
    pub sample_rate: u32,
    /// Device channels; the render callback always produces stereo.
    pub channels: u16,
}

/// A running output stream. Dropping it stops playback.
///
/// The stream lives on its own thread because `cpal` streams cannot move between threads on
/// every platform; this handle is `Send` and can live in any application state.
#[derive(Debug)]
pub struct AudioOutput {
    info: OutputInfo,
    stop: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl AudioOutput {
    /// Opens the default output device and starts calling `render` with interleaved stereo
    /// buffers. `render` runs on the audio thread and must be real-time safe.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError`] if there is no device or it cannot be opened.
    pub fn open_default(
        render: impl FnMut(&mut [f32]) + Send + 'static,
    ) -> Result<Self, AudioError> {
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let thread = thread::Builder::new()
            .name("soundbit-audio".to_owned())
            .spawn(move || {
                match open_stream(render) {
                    Ok((stream, info)) => {
                        let _ = ready_tx.send(Ok(info));
                        // Keep the stream alive until the handle is dropped.
                        let _ = stop_rx.recv();
                        drop(stream);
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            })?;
        let info = ready_rx.recv().map_err(|_| AudioError::ThreadStopped)??;
        Ok(Self {
            info,
            stop: Some(stop_tx),
            thread: Some(thread),
        })
    }

    /// The opened device and format.
    pub fn info(&self) -> &OutputInfo {
        &self.info
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn open_stream(
    render: impl FnMut(&mut [f32]) + Send + 'static,
) -> Result<(Stream, OutputInfo), AudioError> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or(AudioError::NoDevice)?;
    let supported = device.default_output_config()?;
    let config = supported.config();
    let info = OutputInfo {
        device: device
            .description()
            .map_or_else(|_| "default".to_owned(), |d| d.name().to_owned()),
        sample_rate: config.sample_rate,
        channels: config.channels,
    };
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, render)?,
        SampleFormat::F64 => build::<f64>(&device, config, render)?,
        SampleFormat::I16 => build::<i16>(&device, config, render)?,
        SampleFormat::I32 => build::<i32>(&device, config, render)?,
        SampleFormat::U16 => build::<u16>(&device, config, render)?,
        SampleFormat::U8 => build::<u8>(&device, config, render)?,
        SampleFormat::I8 => build::<i8>(&device, config, render)?,
        other => return Err(AudioError::UnsupportedFormat(other.to_string())),
    };
    stream.play()?;
    Ok((stream, info))
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    mut render: impl FnMut(&mut [f32]) + Send + 'static,
) -> Result<Stream, AudioError>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels.max(1));
    // Allocated here, before the stream starts, so the callback never allocates.
    let mut stereo = vec![0.0_f32; CHUNK_FRAMES * 2];
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            // RT-SAFE
            for chunk in data.chunks_mut(CHUNK_FRAMES * channels) {
                let frames = chunk.len() / channels;
                let buffer = &mut stereo[..frames * 2];
                render(buffer);
                write_frames(buffer, chunk, channels);
            }
        },
        |error| eprintln!("soundbit: audio stream error: {error}"),
        None,
    )?;
    Ok(stream)
}
