//! Instruments converted into ready-to-play DSP parameters.

use soundbit_core::{FmOperator, FmVoice, Instrument, NoiseMode, Pan, Voice, Wavetable};
use soundbit_dsp::{AdsrParams, FmParams, PitchModParams, WAVETABLE_LEN, YmOperator};

use crate::shaping::{FilterPatch, UnisonPatch, pitch_params, reverb_send};

/// Highest value of a 4-bit wavetable sample.
const WAVE_LEVEL_MAX: f32 = 15.0;

/// How a voice's sound is generated, with parameters for one sample rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SourcePatch {
    Pulse { duty: f64 },
    Triangle,
    Sawtooth,
    Sine,
    Noise { short: bool },
    Wavetable([f32; WAVETABLE_LEN]),
    Fm(FmParams),
}

/// Left and right gain factors.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct StereoGain {
    pub(crate) left: f32,
    pub(crate) right: f32,
}

impl StereoGain {
    /// Balance panning: the centre plays at full level on both sides, and panning attenuates
    /// only the opposite side.
    fn from_pan(pan: Pan) -> Self {
        let position = f32::from(pan.get()) / 100.0;
        Self {
            left: (1.0 - position).min(1.0),
            right: (1.0 + position).min(1.0),
        }
    }

    pub(crate) fn scaled(self, gain: f32) -> Self {
        Self {
            left: self.left * gain,
            right: self.right * gain,
        }
    }
}

/// An instrument compiled for one sample rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Patch {
    pub(crate) source: SourcePatch,
    pub(crate) envelope: AdsrParams,
    /// Per-channel gain from the instrument's volume and pan.
    pub(crate) gain: StereoGain,
    pub(crate) pitch: PitchModParams,
    pub(crate) unison: UnisonPatch,
    pub(crate) filter: Option<FilterPatch>,
    /// Mono gain into the reverb, including the instrument's volume.
    pub(crate) reverb_send: f32,
}

impl Patch {
    pub(crate) fn compile(instrument: &Instrument, sample_rate: u32) -> Self {
        let envelope = instrument.envelope;
        let volume = f32::from(instrument.volume.get()) / 100.0;
        Self {
            source: source_patch(&instrument.voice, sample_rate),
            envelope: AdsrParams::from_millis(
                envelope.attack.get(),
                envelope.decay.get(),
                f32::from(envelope.sustain.get()) / 100.0,
                envelope.release.get(),
                sample_rate,
            ),
            gain: StereoGain::from_pan(instrument.pan).scaled(volume),
            pitch: pitch_params(instrument.pitch_envelope, instrument.vibrato, sample_rate),
            unison: UnisonPatch::compile(instrument.unison, &instrument.voice),
            filter: FilterPatch::compile(instrument.filter),
            reverb_send: reverb_send(instrument),
        }
    }
}

fn source_patch(voice: &Voice, sample_rate: u32) -> SourcePatch {
    match voice {
        Voice::Pulse { duty } => SourcePatch::Pulse {
            duty: f64::from(duty.fraction()),
        },
        Voice::Triangle => SourcePatch::Triangle,
        Voice::Sawtooth => SourcePatch::Sawtooth,
        Voice::Sine => SourcePatch::Sine,
        Voice::Noise { mode } => SourcePatch::Noise {
            short: *mode == NoiseMode::Short,
        },
        Voice::Wavetable(table) => SourcePatch::Wavetable(normalized(table)),
        Voice::Fm(fm) => SourcePatch::Fm(fm_params(fm, sample_rate)),
    }
}

/// Maps 4-bit samples to `[-1, 1]` and removes the table's DC offset, so notes start and stop
/// without clicks.
fn normalized(table: &Wavetable) -> [f32; WAVETABLE_LEN] {
    let scaled = table
        .samples
        .map(|level| f32::from(level.get()) / WAVE_LEVEL_MAX * 2.0 - 1.0);
    let mean = scaled.iter().sum::<f32>() / WAVETABLE_LEN as f32;
    scaled.map(|sample| (sample - mean).clamp(-1.0, 1.0))
}

fn fm_params(voice: &FmVoice, sample_rate: u32) -> FmParams {
    FmParams::from_ym(
        voice.algorithm.get(),
        voice.feedback.get(),
        &voice.operators.map(ym_operator),
        f64::from(sample_rate),
    )
}

fn ym_operator(op: FmOperator) -> YmOperator {
    YmOperator {
        multiple: op.multiple.get(),
        detune: op.detune.get(),
        total_level: op.total_level.get(),
        attack_rate: op.attack_rate.get(),
        decay_rate: op.decay_rate.get(),
        sustain_rate: op.sustain_rate.get(),
        sustain_level: op.sustain_level.get(),
        release_rate: op.release_rate.get(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_attenuates_the_opposite_side() {
        let centre = StereoGain::from_pan(Pan::default());
        assert_eq!((centre.left, centre.right), (1.0, 1.0));
        let left = StereoGain::from_pan(Pan::MIN);
        assert_eq!((left.left, left.right), (1.0, 0.0));
    }

    #[test]
    fn wavetables_are_centred() {
        let table = normalized(&Wavetable::triangle());
        let mean: f32 = table.iter().sum::<f32>() / WAVETABLE_LEN as f32;
        assert!(mean.abs() < 1e-6);
        assert!(table.iter().all(|sample| (-1.0..=1.0).contains(sample)));
    }
}
