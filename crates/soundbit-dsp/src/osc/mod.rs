//! Oscillators for the basic chip waveforms.

mod blep;
mod noise;
mod pulse;
mod saw;
mod sine;
mod triangle;
mod wavetable;

pub use noise::NoiseOsc;
pub use pulse::PulseOsc;
pub use saw::SawOsc;
pub use sine::SineOsc;
pub use triangle::TriangleOsc;
pub use wavetable::{WAVETABLE_LEN, WavetableOsc};

/// A periodic sound source producing one sample at a time.
pub trait Oscillator {
    /// Sets the pitch in hertz.
    fn set_frequency(&mut self, hz: f64, sample_rate: f64);

    /// Restarts the waveform from the beginning of its cycle.
    fn reset(&mut self);

    /// Jumps to a position in the cycle, in `[0, 1)`. Unison copies start at different
    /// positions so they do not add up to a loud click on the first sample. Sources without a
    /// cycle (noise) ignore it.
    fn set_phase(&mut self, _phase: f64) {}

    /// Produces the next sample, in `[-1, 1]`.
    fn next_sample(&mut self) -> f32;
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::Oscillator;

    /// Counts rising zero crossings per second, averaged over a few seconds and rounded.
    pub(crate) fn measured_frequency(osc: &mut impl Oscillator, hz: f64) -> usize {
        const SAMPLE_RATE: f64 = 48_000.0;
        const SECONDS: usize = 4;
        osc.set_frequency(hz, SAMPLE_RATE);
        osc.reset();
        let mut previous = osc.next_sample();
        let mut crossings = 0;
        for _ in 1..48_000 * SECONDS {
            let sample = osc.next_sample();
            assert!(
                (-1.0..=1.0).contains(&sample),
                "sample {sample} out of range"
            );
            if previous < 0.0 && sample >= 0.0 {
                crossings += 1;
            }
            previous = sample;
        }
        (crossings + SECONDS / 2) / SECONDS
    }
}
