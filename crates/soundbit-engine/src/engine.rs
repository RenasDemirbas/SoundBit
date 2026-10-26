//! The playback engine.

use soundbit_core::limits::MAX_TRACKS;
use soundbit_dsp::Reverb;
use soundbit_dsp::math::soft_clip;

use crate::meter::{Meters, TrackPower};
use crate::schedule::Schedule;
use crate::track::TrackPlayer;

/// Output channels: interleaved stereo.
pub const CHANNELS: u16 = 2;

/// Headroom so several full-level tracks can play at once before the soft clipper engages.
const MASTER_GAIN: f32 = 0.4;

/// Plays a [`Schedule`] and produces interleaved stereo samples.
///
/// Every method marked RT-safe never allocates, locks or performs I/O, so it can run inside an
/// audio callback. Player state for the maximum number of tracks and the reverb's delay lines
/// are allocated up front, which lets [`Engine::replace_schedule`] swap in an edited song
/// while it plays.
#[derive(Debug, Clone)]
pub struct Engine {
    schedule: Schedule,
    players: Vec<TrackPlayer>,
    reverb: Reverb,
    reverb_buffer: Vec<f32>,
    meters: Option<Meters>,
    sample_rate: f64,
    position: u64,
    looping: bool,
}

impl Engine {
    /// Prepares to play `schedule` from the beginning, without looping.
    pub fn new(schedule: Schedule) -> Self {
        let slots = MAX_TRACKS.max(schedule.tracks().len());
        let players = (0..slots).map(|_| TrackPlayer::new()).collect();
        let rate = schedule.sample_rate();
        let reverb = Reverb::new(rate, schedule.reverb());
        Self {
            players,
            reverb,
            reverb_buffer: vec![0.0; Reverb::buffer_len(rate)],
            meters: None,
            sample_rate: f64::from(rate),
            schedule,
            position: 0,
            looping: false,
        }
    }

    /// The schedule being played.
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// Current playback position in samples.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Starts measuring the level of every track, for [`Engine::track_power`]. Used by offline
    /// rendering; live playback leaves it off.
    pub fn enable_metering(&mut self) {
        self.meters = Some(Meters::new(self.players.len()));
    }

    /// Peak and mean power of each track's contribution to the output since metering was
    /// enabled, in track order. Empty without metering.
    pub fn track_power(&self) -> Vec<TrackPower> {
        let tracks = self.schedule.tracks().len();
        self.meters
            .as_ref()
            .map_or_else(Vec::new, |meters| meters.power(tracks))
    }

    /// Repeats the song's loop region, or the whole song if it has none.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Whether the arrangement has ended (when not looping) and every note has finished
    /// releasing.
    pub fn is_finished(&self) -> bool {
        !self.looping
            && self.position >= self.schedule.end_sample()
            && self.reverb.is_quiet()
            && self
                .players
                .iter()
                .zip(self.schedule.tracks())
                .all(|(player, plan)| player.is_idle(plan))
    }

    // RT-SAFE
    /// Jumps to `position`. Sounding notes are released and ring out.
    pub fn seek(&mut self, position: u64) {
        self.position = position;
        for (player, plan) in self.players.iter_mut().zip(self.schedule.tracks()) {
            player.release_all(position);
            player.seek(position, plan);
        }
    }

    // RT-SAFE
    /// Swaps in a new schedule (for example after an edit) at the current position and returns
    /// the old one, which the caller should drop off the audio thread. Sounding notes keep
    /// playing; notes of removed tracks stop.
    pub fn replace_schedule(&mut self, schedule: Schedule) -> Schedule {
        let old = std::mem::replace(&mut self.schedule, schedule);
        self.reverb.set_params(self.schedule.reverb());
        let plans = self.schedule.tracks();
        for (index, player) in self.players.iter_mut().enumerate() {
            match plans.get(index) {
                Some(plan) => player.seek(self.position, plan),
                None => player.reset(),
            }
        }
        old
    }

    // RT-SAFE
    /// Fills `out` with interleaved stereo frames and advances the position. A trailing odd
    /// sample, if any, is set to silence.
    pub fn process(&mut self, out: &mut [f32]) {
        let mut frames = out.chunks_exact_mut(usize::from(CHANNELS));
        for frame in &mut frames {
            if let Some((start, end)) = self.loop_bounds()
                && self.position >= end
            {
                self.seek(start);
            }
            let (left, right) = self.mix_frame();
            frame[0] = soft_clip(left * MASTER_GAIN);
            frame[1] = soft_clip(right * MASTER_GAIN);
            self.position += 1;
        }
        frames.into_remainder().fill(0.0);
    }

    /// The region to repeat, if looping and the song is not empty.
    fn loop_bounds(&self) -> Option<(u64, u64)> {
        if !self.looping {
            return None;
        }
        self.schedule
            .loop_region()
            .or(Some((0, self.schedule.end_sample())))
            .filter(|&(start, end)| start < end)
    }

    // RT-SAFE
    fn mix_frame(&mut self) -> (f32, f32) {
        let patches = self.schedule.patches();
        let (mut left, mut right, mut send) = (0.0, 0.0, 0.0);
        for (index, (player, plan)) in self
            .players
            .iter_mut()
            .zip(self.schedule.tracks())
            .enumerate()
        {
            let frame = player.render(self.position, plan, patches, self.sample_rate);
            if let Some(meters) = &mut self.meters {
                meters.record(index, frame.left * MASTER_GAIN, frame.right * MASTER_GAIN);
            }
            left += frame.left;
            right += frame.right;
            send += frame.send;
        }
        // Skipping a silent reverb keeps songs without sends bit-identical to a dry mix.
        if self.schedule.uses_reverb() || !self.reverb.is_quiet() {
            let (wet_left, wet_right) = self.reverb.process(&mut self.reverb_buffer, send);
            left += wet_left;
            right += wet_right;
        }
        (left, right)
    }
}
