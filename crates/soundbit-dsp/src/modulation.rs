//! Pitch modulation: a sweep at the start of a note plus vibrato.

use crate::Phase;
use crate::math::sine;

/// Pitch modulation settings for one sample rate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PitchModParams {
    /// Offset in semitones at the start of the note.
    pub sweep_semitones: f64,
    /// Length of the glide from the offset to the note's pitch, in samples.
    pub sweep_samples: u32,
    /// Vibrato depth in semitones either side of the note.
    pub vibrato_semitones: f64,
    /// Vibrato frequency in hertz.
    pub vibrato_hz: f64,
    /// Samples before the vibrato starts.
    pub vibrato_delay: u32,
}

impl PitchModParams {
    /// Converts musical units: `sweep_ms` and `delay_ms` in milliseconds, `depth_cents` in
    /// hundredths of a semitone.
    pub fn new(
        sweep_semitones: f64,
        sweep_ms: u16,
        depth_cents: f64,
        vibrato_hz: f64,
        delay_ms: u16,
        sample_rate: u32,
    ) -> Self {
        let samples = |ms: u16| {
            let total = u64::from(ms) * u64::from(sample_rate) / 1_000;
            u32::try_from(total).unwrap_or(u32::MAX)
        };
        let sweep_samples = samples(sweep_ms);
        Self {
            sweep_semitones: if sweep_samples == 0 {
                0.0
            } else {
                sweep_semitones
            },
            sweep_samples,
            vibrato_semitones: depth_cents / 100.0,
            vibrato_hz,
            vibrato_delay: samples(delay_ms),
        }
    }

    /// Whether the pitch never moves, so the oscillator frequency can be set once.
    pub fn is_static(&self) -> bool {
        self.sweep_semitones == 0.0 && self.vibrato_semitones == 0.0
    }
}

/// The running modulation of one note.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PitchMod {
    params: PitchModParams,
    elapsed: u32,
    lfo: Phase,
}

impl PitchMod {
    /// Starts modulating a new note.
    pub fn start(&mut self, params: PitchModParams, sample_rate: f64) {
        self.params = params;
        self.elapsed = 0;
        self.lfo.set_frequency(params.vibrato_hz, sample_rate);
        self.lfo.reset();
    }

    /// Returns the current pitch offset in semitones and advances by one sample.
    pub fn next_offset(&mut self) -> f64 {
        let params = &self.params;
        let mut offset = 0.0;
        if self.elapsed < params.sweep_samples {
            let remaining = 1.0 - f64::from(self.elapsed) / f64::from(params.sweep_samples);
            offset += params.sweep_semitones * remaining;
        }
        if params.vibrato_semitones != 0.0 && self.elapsed >= params.vibrato_delay {
            // The LFO starts at a zero crossing, so the vibrato fades in without a jump.
            offset += params.vibrato_semitones * sine(self.lfo.tick());
        }
        self.elapsed = self.elapsed.saturating_add(1);
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 48_000;

    fn modulator(params: PitchModParams) -> PitchMod {
        let mut pitch = PitchMod::default();
        pitch.start(params, f64::from(SAMPLE_RATE));
        pitch
    }

    #[test]
    fn sweep_glides_linearly_to_the_note() {
        let mut pitch = modulator(PitchModParams::new(24.0, 1, 0.0, 5.0, 0, SAMPLE_RATE));
        let offsets: std::vec::Vec<f64> = (0..50).map(|_| pitch.next_offset()).collect();
        assert_eq!(offsets[0], 24.0);
        assert!((offsets[24] - 12.0).abs() < 1e-9);
        assert_eq!(offsets[48], 0.0);
    }

    #[test]
    fn vibrato_waits_then_wobbles_symmetrically() {
        let mut pitch = modulator(PitchModParams::new(0.0, 0, 50.0, 6.0, 100, SAMPLE_RATE));
        let offsets: std::vec::Vec<f64> = (0..48_000).map(|_| pitch.next_offset()).collect();
        assert!(offsets[..4_800].iter().all(|&o| o == 0.0));
        let max = offsets.iter().copied().fold(f64::MIN, f64::max);
        let min = offsets.iter().copied().fold(f64::MAX, f64::min);
        assert!(
            (max - 0.5).abs() < 1e-3 && (min + 0.5).abs() < 1e-3,
            "{min}..{max}"
        );
    }

    #[test]
    fn zero_length_sweep_is_static() {
        assert!(PitchModParams::new(12.0, 0, 0.0, 5.0, 0, SAMPLE_RATE).is_static());
        assert!(!PitchModParams::new(0.0, 0, 10.0, 5.0, 0, SAMPLE_RATE).is_static());
    }
}
