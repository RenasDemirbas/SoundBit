//! A stereo room reverb.
//!
//! The structure follows Jezar's public-domain Freeverb: eight damped feedback comb filters in
//! parallel, then four allpass filters in series, per channel. The right channel's delay lines
//! are slightly longer, which spreads the sound across the stereo field.
//!
//! The crate cannot allocate, so the delay lines live in one caller-owned buffer of
//! [`Reverb::buffer_len`] samples, allocated once off the audio thread.

/// Comb filter lengths in samples at 44.1 kHz.
const COMB_TUNING: [usize; COMBS] = [1_116, 1_188, 1_277, 1_356, 1_422, 1_491, 1_557, 1_617];

/// Allpass filter lengths in samples at 44.1 kHz.
const ALLPASS_TUNING: [usize; ALLPASSES] = [556, 441, 341, 225];

/// Extra delay of the right channel at 44.1 kHz.
const STEREO_SPREAD: usize = 23;

/// Sample rate the tunings were designed for.
const TUNING_RATE: u64 = 44_100;

const COMBS: usize = 8;
const ALLPASSES: usize = 4;
const LINES: usize = 2 * (COMBS + ALLPASSES);

/// Input attenuation that keeps eight summed combs from overloading.
const INPUT_GAIN: f32 = 0.015;

/// Output gain, so a fully sent sound comes back at a comparable level.
const WET_GAIN: f32 = 3.0;

/// Comb feedback range: room size 0 maps to the offset, 1 to offset + scale.
const ROOM_OFFSET: f32 = 0.7;
const ROOM_SCALE: f32 = 0.28;

/// Damping range of the combs' low-pass filters.
const DAMP_SCALE: f32 = 0.4;

/// Allpass feedback.
const ALLPASS_FEEDBACK: f32 = 0.5;

/// Output below this level counts as silence.
const QUIET_LEVEL: f32 = 1e-5;

/// Reverb character for [`Reverb::set_params`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReverbParams {
    /// Room size, `0` (small) to `1` (large hall).
    pub size: f32,
    /// High-frequency damping, `0` (bright) to `1` (dark).
    pub damping: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Line {
    start: usize,
    len: usize,
    pos: usize,
    /// Comb filters only: state of the damping low-pass.
    filter: f32,
}

impl Line {
    /// Reads the oldest sample and writes `value` in its place.
    fn swap(&mut self, buffer: &mut [f32], value: f32) -> f32 {
        let Some(slot) = buffer.get_mut(self.start + self.pos) else {
            return 0.0;
        };
        let out = *slot;
        *slot = value;
        self.pos += 1;
        if self.pos >= self.len {
            self.pos = 0;
        }
        out
    }

    fn peek(&self, buffer: &[f32]) -> f32 {
        buffer.get(self.start + self.pos).copied().unwrap_or(0.0)
    }

    fn comb(&mut self, buffer: &mut [f32], input: f32, feedback: f32, damp: f32) -> f32 {
        let out = self.peek(buffer);
        self.filter = flush(out * (1.0 - damp) + self.filter * damp);
        self.swap(buffer, input + self.filter * feedback);
        out
    }

    fn allpass(&mut self, buffer: &mut [f32], input: f32) -> f32 {
        let delayed = self.peek(buffer);
        self.swap(buffer, flush(input + delayed * ALLPASS_FEEDBACK));
        delayed - input
    }
}

/// Freeverb's stereo reverb. Real-time safe: processing only touches the caller's buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct Reverb {
    lines: [Line; LINES],
    feedback: f32,
    damp: f32,
    quiet_samples: u32,
    quiet_after: u32,
}

impl Reverb {
    /// Length of the buffer [`Reverb::process`] needs at `sample_rate`.
    pub fn buffer_len(sample_rate: u32) -> usize {
        layout(sample_rate).iter().map(|line| line.len).sum()
    }

    /// A reverb for `sample_rate`. The buffer passed to [`Reverb::process`] must start out
    /// silent (all zeros).
    pub fn new(sample_rate: u32, params: ReverbParams) -> Self {
        let mut reverb = Self {
            lines: layout(sample_rate),
            feedback: 0.0,
            damp: 0.0,
            // The buffer starts silent, so the reverb starts quiet.
            quiet_samples: sample_rate / 20,
            // A twentieth of a second of near-silence means the tail has died away.
            quiet_after: sample_rate / 20,
        };
        reverb.set_params(params);
        reverb
    }

    /// Changes the room without clearing the sound that is ringing.
    pub fn set_params(&mut self, params: ReverbParams) {
        self.feedback = ROOM_OFFSET + ROOM_SCALE * params.size.clamp(0.0, 1.0);
        self.damp = DAMP_SCALE * params.damping.clamp(0.0, 1.0);
    }

    /// Whether the tail has died away, so skipping [`Reverb::process`] changes nothing
    /// audible.
    pub fn is_quiet(&self) -> bool {
        self.quiet_samples >= self.quiet_after
    }

    /// Feeds one mono sample in and returns one stereo frame of reverb.
    pub fn process(&mut self, buffer: &mut [f32], input: f32) -> (f32, f32) {
        let input = input * INPUT_GAIN;
        let (left_lines, right_lines) = self.lines.split_at_mut(COMBS + ALLPASSES);
        let left = channel(left_lines, buffer, input, self.feedback, self.damp) * WET_GAIN;
        let right = channel(right_lines, buffer, input, self.feedback, self.damp) * WET_GAIN;
        if left.abs() < QUIET_LEVEL && right.abs() < QUIET_LEVEL && input == 0.0 {
            self.quiet_samples = self.quiet_samples.saturating_add(1);
        } else {
            self.quiet_samples = 0;
        }
        (left, right)
    }
}

fn channel(lines: &mut [Line], buffer: &mut [f32], input: f32, feedback: f32, damp: f32) -> f32 {
    let (combs, allpasses) = lines.split_at_mut(COMBS);
    let mut out = 0.0;
    for comb in combs {
        out += comb.comb(buffer, input, feedback, damp);
    }
    for allpass in allpasses {
        out = allpass.allpass(buffer, out);
    }
    out
}

/// Delay lines scaled to `sample_rate`, packed one after another into the shared buffer.
fn layout(sample_rate: u32) -> [Line; LINES] {
    let scaled = |samples: usize| {
        let len = samples as u64 * u64::from(sample_rate) / TUNING_RATE;
        usize::try_from(len).unwrap_or(usize::MAX).max(1)
    };
    let mut lines = [Line::default(); LINES];
    let mut start = 0;
    for (index, line) in lines.iter_mut().enumerate() {
        let channel_offset = if index >= COMBS + ALLPASSES {
            STEREO_SPREAD
        } else {
            0
        };
        let tuning = match index % (COMBS + ALLPASSES) {
            i if i < COMBS => COMB_TUNING[i],
            i => ALLPASS_TUNING[i - COMBS],
        };
        line.start = start;
        line.len = scaled(tuning + channel_offset);
        start += line.len;
    }
    lines
}

fn flush(value: f32) -> f32 {
    if value.abs() < 1e-25 { 0.0 } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    const SAMPLE_RATE: u32 = 48_000;
    const HALL: ReverbParams = ReverbParams {
        size: 0.8,
        damping: 0.3,
    };

    #[test]
    fn buffer_scales_with_sample_rate() {
        let at_44 = Reverb::buffer_len(44_100);
        assert_eq!(at_44, 2 * (11_024 + 1_563) + 12 * STEREO_SPREAD);
        assert!(Reverb::buffer_len(96_000) > 2 * at_44);
    }

    #[test]
    fn an_impulse_rings_then_dies_away() {
        let mut buffer = vec![0.0; Reverb::buffer_len(SAMPLE_RATE)];
        let mut reverb = Reverb::new(SAMPLE_RATE, HALL);
        let mut energy = [0.0_f32; 2];
        reverb.process(&mut buffer, 1.0);
        for i in 0..SAMPLE_RATE * 20 {
            let (left, right) = reverb.process(&mut buffer, 0.0);
            assert!(left.is_finite() && right.is_finite());
            if i < SAMPLE_RATE {
                energy[0] += left * left;
                energy[1] += right * right;
            }
            if reverb.is_quiet() {
                break;
            }
        }
        assert!(energy[0] > 0.0 && energy[1] > 0.0, "{energy:?}");
        assert_ne!(energy[0], energy[1], "channels should differ");
        assert!(reverb.is_quiet(), "tail never ended");
    }

    #[test]
    fn larger_rooms_ring_longer() {
        let tail = |size: f32| {
            let mut buffer = vec![0.0; Reverb::buffer_len(SAMPLE_RATE)];
            let mut reverb = Reverb::new(SAMPLE_RATE, ReverbParams { size, damping: 0.5 });
            reverb.process(&mut buffer, 1.0);
            (0..SAMPLE_RATE * 30)
                .take_while(|_| {
                    reverb.process(&mut buffer, 0.0);
                    !reverb.is_quiet()
                })
                .count()
        };
        assert!(tail(1.0) > tail(0.0) * 2);
    }

    #[test]
    fn short_buffers_do_not_panic() {
        let mut buffer = vec![0.0; 10];
        let mut reverb = Reverb::new(SAMPLE_RATE, HALL);
        for _ in 0..1_000 {
            reverb.process(&mut buffer, 0.5);
        }
    }
}
