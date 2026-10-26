//! Sound shaping compiled for playback: unison layers, pitch modulation and the filter.

use soundbit_core::{
    Filter, FilterMode, Instrument, PitchEnvelope, Unison, UnisonVoices, Vibrato, Voice,
};
use soundbit_dsp::math::semitone_ratio;
use soundbit_dsp::{FilterResponse, PitchModParams, Svf};

/// Most copies of a voice that play per note.
#[allow(clippy::cast_lossless, reason = "`usize::from` is not const")]
pub(crate) const MAX_UNISON: usize = UnisonVoices::MAX.get() as usize;

/// Gain per layer, `1 / sqrt(layers)`, so unison adds width without getting much louder.
const UNISON_GAIN: [f32; MAX_UNISON] = [1.0, core::f32::consts::FRAC_1_SQRT_2, 0.577_350_3, 0.5];

/// Cents per semitone.
const CENTS: f64 = 100.0;

/// The detuned copies a note plays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UnisonPatch {
    pub(crate) layers: usize,
    /// Frequency ratio of each layer relative to the note.
    pub(crate) ratios: [f64; MAX_UNISON],
    /// Starting point of each layer in its cycle.
    pub(crate) phases: [f64; MAX_UNISON],
    pub(crate) gain: f32,
}

impl UnisonPatch {
    /// Layers spread evenly from `-detune` to `+detune` cents. Noise has no pitch to detune,
    /// so it always plays a single layer.
    pub(crate) fn compile(unison: Unison, voice: &Voice) -> Self {
        let layers = if matches!(voice, Voice::Noise { .. }) {
            1
        } else {
            usize::from(unison.voices.get()).clamp(1, MAX_UNISON)
        };
        let detune = f64::from(unison.detune.get());
        let mut ratios = [1.0; MAX_UNISON];
        let mut phases = [0.0; MAX_UNISON];
        if layers > 1 {
            let last = (layers - 1) as f64;
            for layer in 0..layers {
                let position = 2.0 * layer as f64 / last - 1.0;
                ratios[layer] = semitone_ratio(position * detune / CENTS);
                phases[layer] = layer as f64 / layers as f64;
            }
        }
        Self {
            layers,
            ratios,
            phases,
            gain: UNISON_GAIN[layers - 1],
        }
    }
}

/// Pitch sweep and vibrato for one sample rate.
pub(crate) fn pitch_params(
    sweep: PitchEnvelope,
    vibrato: Vibrato,
    sample_rate: u32,
) -> PitchModParams {
    let sweep_semitones = if sweep.is_off() {
        0.0
    } else {
        f64::from(sweep.semitones.get())
    };
    let depth = if vibrato.is_off() {
        0.0
    } else {
        f64::from(vibrato.depth.get())
    };
    PitchModParams::new(
        sweep_semitones,
        sweep.time.get(),
        depth,
        f64::from(vibrato.rate.get()) / 10.0,
        vibrato.delay.get(),
        sample_rate,
    )
}

/// A filter setting ready to play.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FilterPatch {
    response: FilterResponse,
    resonance: f64,
    cutoff_hz: f64,
    envelope_semitones: f64,
}

impl FilterPatch {
    /// `None` when the filter is off, so unfiltered voices skip it entirely.
    pub(crate) fn compile(filter: Filter) -> Option<Self> {
        let response = match filter.mode {
            FilterMode::Off => return None,
            FilterMode::LowPass => FilterResponse::LowPass,
            FilterMode::HighPass => FilterResponse::HighPass,
            FilterMode::BandPass => FilterResponse::BandPass,
        };
        Some(Self {
            response,
            resonance: f64::from(filter.resonance.get()) / 100.0,
            cutoff_hz: f64::from(filter.cutoff.get()),
            envelope_semitones: f64::from(filter.envelope_depth.get()),
        })
    }
}

/// The filter of one playing note.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct VoiceFilter {
    patch: Option<FilterPatch>,
    svf: Svf,
    sample_rate: f64,
}

impl VoiceFilter {
    // RT-SAFE
    pub(crate) fn start(&mut self, patch: Option<FilterPatch>, sample_rate: f64) {
        self.patch = patch;
        self.sample_rate = sample_rate;
        if let Some(patch) = patch {
            self.svf.set_shape(patch.response, patch.resonance);
            self.svf.set_cutoff(patch.cutoff_hz, sample_rate);
            self.svf.reset();
        }
    }

    // RT-SAFE
    /// Filters one sample; `envelope` is the amplitude envelope's level, which moves the
    /// cutoff by the patch's envelope depth.
    pub(crate) fn process(&mut self, sample: f32, envelope: f32) -> f32 {
        let Some(patch) = self.patch else {
            return sample;
        };
        if patch.envelope_semitones != 0.0 {
            let shift = patch.envelope_semitones * f64::from(envelope);
            self.svf
                .set_cutoff(patch.cutoff_hz * semitone_ratio(shift), self.sample_rate);
        }
        self.svf.process(sample)
    }
}

/// Share of an instrument's sound sent to the reverb, including its volume.
pub(crate) fn reverb_send(instrument: &Instrument) -> f32 {
    f32::from(instrument.reverb_send.get()) / 100.0 * f32::from(instrument.volume.get()) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundbit_core::{NoiseMode, UnisonDetune};

    fn unison(voices: u8, detune: u8) -> Unison {
        Unison {
            voices: UnisonVoices::new(voices).expect("in range"),
            detune: UnisonDetune::new(detune).expect("in range"),
        }
    }

    #[test]
    fn single_layer_is_untouched() {
        let patch = UnisonPatch::compile(Unison::default(), &Voice::Sawtooth);
        assert_eq!(patch.layers, 1);
        assert_eq!(
            (patch.ratios[0], patch.phases[0], patch.gain),
            (1.0, 0.0, 1.0)
        );
    }

    #[test]
    fn layers_spread_symmetrically() {
        let patch = UnisonPatch::compile(unison(3, 50), &Voice::Sawtooth);
        assert_eq!(patch.layers, 3);
        assert!((patch.ratios[0] * patch.ratios[2] - 1.0).abs() < 1e-12);
        assert_eq!(patch.ratios[1], 1.0);
        assert!((patch.ratios[2] - semitone_ratio(0.5)).abs() < 1e-12);
    }

    #[test]
    fn noise_ignores_unison() {
        let noise = Voice::Noise {
            mode: NoiseMode::Long,
        };
        assert_eq!(UnisonPatch::compile(unison(4, 20), &noise).layers, 1);
    }

    #[test]
    fn filter_off_compiles_to_nothing() {
        assert_eq!(FilterPatch::compile(Filter::default()), None);
    }
}
