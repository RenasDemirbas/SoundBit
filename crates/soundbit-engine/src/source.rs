//! The sound generator of one playing voice.

use soundbit_dsp::{
    FmSynth, NoiseOsc, Oscillator, PulseOsc, SawOsc, SineOsc, TriangleOsc, WavetableOsc,
};

use crate::patch::SourcePatch;

/// One of the DSP generators. Lives inline in the voice, so starting a note never allocates.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "boxing the FM variant would allocate on the audio thread; voices are preallocated"
)]
pub(crate) enum Source {
    #[default]
    Silent,
    Pulse(PulseOsc),
    Triangle(TriangleOsc),
    Sawtooth(SawOsc),
    Sine(SineOsc),
    Noise(NoiseOsc),
    Wavetable(WavetableOsc),
    Fm(FmSynth),
}

fn tuned<O: Oscillator>(mut osc: O, hz: f64, sample_rate: f64) -> O {
    osc.set_frequency(hz, sample_rate);
    osc.reset();
    osc
}

impl Source {
    // RT-SAFE
    pub(crate) fn start(&mut self, patch: &SourcePatch, hz: f64, sample_rate: f64) {
        *self = match *patch {
            SourcePatch::Pulse { duty } => Self::Pulse(tuned(PulseOsc::new(duty), hz, sample_rate)),
            SourcePatch::Triangle => Self::Triangle(tuned(TriangleOsc::default(), hz, sample_rate)),
            SourcePatch::Sawtooth => Self::Sawtooth(tuned(SawOsc::default(), hz, sample_rate)),
            SourcePatch::Sine => Self::Sine(tuned(SineOsc::default(), hz, sample_rate)),
            SourcePatch::Noise { short } => {
                Self::Noise(tuned(NoiseOsc::new(short), hz, sample_rate))
            }
            SourcePatch::Wavetable(table) => {
                Self::Wavetable(tuned(WavetableOsc::new(table), hz, sample_rate))
            }
            SourcePatch::Fm(params) => {
                let mut synth = FmSynth::default();
                synth.note_on(&params, hz, sample_rate);
                Self::Fm(synth)
            }
        };
    }

    // RT-SAFE
    /// Starts a note at a given point of the waveform's cycle (unison layers).
    pub(crate) fn start_at(&mut self, patch: &SourcePatch, hz: f64, sample_rate: f64, phase: f64) {
        self.start(patch, hz, sample_rate);
        if phase != 0.0 {
            self.set_phase(phase);
        }
    }

    // RT-SAFE
    /// Changes the pitch of the sounding note without restarting it.
    pub(crate) fn set_frequency(&mut self, hz: f64, sample_rate: f64) {
        match self {
            Self::Silent => {}
            Self::Pulse(osc) => osc.set_frequency(hz, sample_rate),
            Self::Triangle(osc) => osc.set_frequency(hz, sample_rate),
            Self::Sawtooth(osc) => osc.set_frequency(hz, sample_rate),
            Self::Sine(osc) => osc.set_frequency(hz, sample_rate),
            Self::Noise(osc) => osc.set_frequency(hz, sample_rate),
            Self::Wavetable(osc) => osc.set_frequency(hz, sample_rate),
            Self::Fm(synth) => synth.set_frequency(hz, sample_rate),
        }
    }

    // RT-SAFE
    fn set_phase(&mut self, phase: f64) {
        match self {
            Self::Pulse(osc) => osc.set_phase(phase),
            Self::Triangle(osc) => osc.set_phase(phase),
            Self::Sawtooth(osc) => osc.set_phase(phase),
            Self::Sine(osc) => osc.set_phase(phase),
            Self::Wavetable(osc) => osc.set_phase(phase),
            // Noise has no cycle; FM operators keep their own phases.
            Self::Silent | Self::Noise(_) | Self::Fm(_) => {}
        }
    }

    // RT-SAFE
    pub(crate) fn release(&mut self) {
        if let Self::Fm(synth) = self {
            synth.note_off();
        }
    }

    /// Whether the source shapes its own amplitude (FM operator envelopes) instead of using the
    /// instrument's ADSR.
    pub(crate) fn has_own_envelope(&self) -> bool {
        matches!(self, Self::Fm(_))
    }

    /// Whether a self-enveloped source has gone quiet.
    pub(crate) fn is_silent(&self) -> bool {
        match self {
            Self::Silent => true,
            Self::Fm(synth) => synth.is_silent(),
            _ => false,
        }
    }

    // RT-SAFE
    pub(crate) fn next_sample(&mut self) -> f32 {
        match self {
            Self::Silent => 0.0,
            Self::Pulse(osc) => osc.next_sample(),
            Self::Triangle(osc) => osc.next_sample(),
            Self::Sawtooth(osc) => osc.next_sample(),
            Self::Sine(osc) => osc.next_sample(),
            Self::Noise(osc) => osc.next_sample(),
            Self::Wavetable(osc) => osc.next_sample(),
            Self::Fm(synth) => synth.next_sample(),
        }
    }
}
