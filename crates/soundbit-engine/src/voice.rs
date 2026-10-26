//! A single sounding note.

use soundbit_dsp::math::{note_frequency, semitone_ratio};
use soundbit_dsp::{Adsr, PitchMod};

use crate::patch::{Patch, StereoGain};
use crate::schedule::NoteEvent;
use crate::shaping::{MAX_UNISON, VoiceFilter};
use crate::source::Source;

/// One rendered frame of a voice: the stereo signal plus its mono reverb send.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct VoiceFrame {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) send: f32,
}

/// One playing note: its unison layers, envelope, modulation, filter and gains.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Voice {
    layers: [Source; MAX_UNISON],
    layer_count: usize,
    layer_ratios: [f64; MAX_UNISON],
    layer_gain: f32,
    hz: f64,
    sample_rate: f64,
    /// Whether the pitch moves, so the layers must be retuned every sample.
    modulated: bool,
    pitch: PitchMod,
    filter: VoiceFilter,
    envelope: Adsr,
    gain: StereoGain,
    send: f32,
    started_at: u64,
    release_at: u64,
    released: bool,
    active: bool,
}

impl Voice {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn started_at(&self) -> u64 {
        self.started_at
    }

    // RT-SAFE
    pub(crate) fn start(&mut self, patch: &Patch, event: &NoteEvent, sample_rate: f64) {
        let unison = &patch.unison;
        self.hz = note_frequency(event.pitch);
        self.sample_rate = sample_rate;
        self.layer_count = unison.layers.clamp(1, MAX_UNISON);
        self.layer_ratios = unison.ratios;
        self.layer_gain = unison.gain;
        for ((layer, ratio), phase) in self
            .layers
            .iter_mut()
            .zip(unison.ratios)
            .zip(unison.phases)
            .take(self.layer_count)
        {
            layer.start_at(&patch.source, self.hz * ratio, sample_rate, phase);
        }
        self.modulated = !patch.pitch.is_static();
        self.pitch.start(patch.pitch, sample_rate);
        self.filter.start(patch.filter, sample_rate);
        self.envelope.set_params(patch.envelope);
        self.envelope.note_on();
        self.gain = patch.gain.scaled(event.velocity);
        self.send = patch.reverb_send * event.velocity;
        self.started_at = event.on;
        self.release_at = event.off;
        self.released = false;
        self.active = true;
    }

    // RT-SAFE
    /// Makes the note release no later than `now`.
    pub(crate) fn release_by(&mut self, now: u64) {
        self.release_at = self.release_at.min(now);
    }

    // RT-SAFE
    /// Renders one frame at sample position `now`.
    pub(crate) fn render(&mut self, now: u64) -> VoiceFrame {
        if !self.active {
            return VoiceFrame::default();
        }
        if !self.released && now >= self.release_at {
            self.released = true;
            self.envelope.note_off();
            for layer in self.layers.iter_mut().take(self.layer_count) {
                layer.release();
            }
        }

        if self.modulated {
            self.retune();
        }
        let sample = self.layers_sample();
        let envelope = self.envelope.next_level();
        let sample = self.filter.process(sample, envelope);
        let first = &self.layers[0];
        let (level, finished) = if first.has_own_envelope() {
            (1.0, first.is_silent())
        } else {
            (envelope, self.envelope.is_idle())
        };
        if finished {
            self.active = false;
        }
        let out = sample * level;
        VoiceFrame {
            left: out * self.gain.left,
            right: out * self.gain.right,
            send: out * self.send,
        }
    }

    // RT-SAFE
    fn retune(&mut self) {
        let hz = self.hz * semitone_ratio(self.pitch.next_offset());
        for (layer, ratio) in self
            .layers
            .iter_mut()
            .zip(self.layer_ratios)
            .take(self.layer_count)
        {
            layer.set_frequency(hz * ratio, self.sample_rate);
        }
    }

    // RT-SAFE
    fn layers_sample(&mut self) -> f32 {
        if self.layer_count == 1 {
            return self.layers[0].next_sample();
        }
        let sum: f32 = self
            .layers
            .iter_mut()
            .take(self.layer_count)
            .map(Source::next_sample)
            .sum();
        sum * self.layer_gain
    }
}
