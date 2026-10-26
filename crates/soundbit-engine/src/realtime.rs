//! Live playback: an audio-thread player controlled from other threads.
//!
//! [`player`] returns two halves connected by lock-free single-producer, single-consumer
//! queues. [`RtPlayer`] lives inside the audio callback; [`PlayerControl`] lives anywhere else.
//! Engines and schedules are built on the control side and moved to the audio thread; replaced
//! ones are moved back so they are freed off the audio thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use rtrb::{Consumer, Producer, RingBuffer};
use thiserror::Error;

use crate::engine::Engine;
use crate::schedule::Schedule;

/// Messages queued before the audio thread must have caught up.
const QUEUE_CAPACITY: usize = 64;

/// The audio thread did not take a message in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("the audio thread is not responding; is the audio device running?")]
pub struct PlayerError;

enum ToAudio {
    Start(Box<Engine>),
    Replace(Schedule),
    Stop,
}

/// Objects the audio thread hands back so they are dropped elsewhere.
#[allow(
    dead_code,
    reason = "the payload is only held so it is dropped on the control thread"
)]
enum Garbage {
    Engine(Box<Engine>),
    Schedule(Schedule),
}

#[derive(Debug, Default)]
struct Status {
    position: AtomicU64,
    playing: AtomicBool,
}

/// Creates a connected control and audio-thread player.
pub fn player() -> (PlayerControl, RtPlayer) {
    let (to_audio, inbox) = RingBuffer::new(QUEUE_CAPACITY);
    let (garbage_out, garbage_in) = RingBuffer::new(QUEUE_CAPACITY);
    let status = Arc::new(Status::default());
    let control = PlayerControl {
        outbox: to_audio,
        garbage: garbage_in,
        status: Arc::clone(&status),
    };
    let rt = RtPlayer {
        inbox,
        garbage: garbage_out,
        status,
        engine: None,
        playing: false,
    };
    (control, rt)
}

/// Controls playback from a non-audio thread.
pub struct PlayerControl {
    outbox: Producer<ToAudio>,
    garbage: Consumer<Garbage>,
    status: Arc<Status>,
}

impl PlayerControl {
    /// Starts playing `schedule` from sample `from`, replacing whatever was playing.
    ///
    /// # Errors
    ///
    /// Returns [`PlayerError`] if the audio thread is not draining its queue.
    pub fn start(
        &mut self,
        schedule: Schedule,
        from: u64,
        looping: bool,
    ) -> Result<(), PlayerError> {
        let mut engine = Engine::new(schedule);
        engine.set_looping(looping);
        engine.seek(from);
        self.status.position.store(from, Ordering::Relaxed);
        self.send(ToAudio::Start(Box::new(engine)))?;
        self.status.playing.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Swaps in an updated schedule at the current position, if something is playing.
    ///
    /// # Errors
    ///
    /// Returns [`PlayerError`] if the audio thread is not draining its queue.
    pub fn update(&mut self, schedule: Schedule) -> Result<(), PlayerError> {
        self.send(ToAudio::Replace(schedule))
    }

    /// Stops playback.
    ///
    /// # Errors
    ///
    /// Returns [`PlayerError`] if the audio thread is not draining its queue.
    pub fn stop(&mut self) -> Result<(), PlayerError> {
        self.status.playing.store(false, Ordering::Relaxed);
        self.send(ToAudio::Stop)
    }

    /// Whether something is playing.
    pub fn is_playing(&self) -> bool {
        self.status.playing.load(Ordering::Relaxed)
    }

    /// Current playback position in samples.
    pub fn position(&self) -> u64 {
        self.status.position.load(Ordering::Relaxed)
    }

    /// Frees objects the audio thread has handed back. Called by every other method too.
    pub fn collect_garbage(&mut self) {
        while self.garbage.pop().is_ok() {}
    }

    fn send(&mut self, message: ToAudio) -> Result<(), PlayerError> {
        self.collect_garbage();
        self.outbox.push(message).map_err(|_| PlayerError)
    }
}

/// The audio-thread half. Call [`RtPlayer::process`] from the audio callback.
pub struct RtPlayer {
    inbox: Consumer<ToAudio>,
    garbage: Producer<Garbage>,
    status: Arc<Status>,
    engine: Option<Box<Engine>>,
    playing: bool,
}

impl RtPlayer {
    // RT-SAFE
    /// Applies pending control messages and fills `out` with interleaved stereo frames.
    pub fn process(&mut self, out: &mut [f32]) {
        while let Ok(message) = self.inbox.pop() {
            self.handle(message);
        }
        match &mut self.engine {
            Some(engine) if self.playing => {
                engine.process(out);
                self.status
                    .position
                    .store(engine.position(), Ordering::Relaxed);
                if engine.is_finished() {
                    self.playing = false;
                    self.status.playing.store(false, Ordering::Relaxed);
                }
            }
            _ => out.fill(0.0),
        }
    }

    // RT-SAFE
    fn handle(&mut self, message: ToAudio) {
        match message {
            ToAudio::Start(engine) => {
                if let Some(old) = self.engine.replace(engine) {
                    self.discard(Garbage::Engine(old));
                }
                self.playing = true;
            }
            ToAudio::Replace(schedule) => match &mut self.engine {
                Some(engine) => {
                    let old = engine.replace_schedule(schedule);
                    self.discard(Garbage::Schedule(old));
                }
                None => self.discard(Garbage::Schedule(schedule)),
            },
            ToAudio::Stop => self.playing = false,
        }
    }

    // RT-SAFE
    fn discard(&mut self, garbage: Garbage) {
        // The control side empties this queue on every call, so it is only full if the
        // controller stopped polling; then dropping here is the lesser evil.
        let _ = self.garbage.push(garbage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundbit_core::{Instrument, Note, Pattern, Song, TrackSettings, Voice};

    fn one_bar_song() -> Song {
        let mut song = Song::new("Live");
        let lead = song
            .add_instrument(Instrument::new("Lead", Voice::Sine))
            .expect("room");
        let mut pattern = Pattern::new("A", lead);
        let note = Note::new("A4".parse().expect("valid"), 0, 16).expect("valid");
        pattern.insert_note(note).expect("empty");
        let pattern = song.add_pattern(pattern).expect("valid");
        let track = song.add_track(TrackSettings::new("T")).expect("room");
        song.place_clip(track, 0, pattern).expect("valid");
        song
    }

    fn schedule() -> Schedule {
        Schedule::compile(&one_bar_song(), 8_000).expect("valid rate")
    }

    fn peak(rt: &mut RtPlayer, frames: usize) -> f32 {
        let mut block = vec![0.0; frames * 2];
        rt.process(&mut block);
        block.iter().fold(0.0, |peak, s| peak.max(s.abs()))
    }

    #[test]
    fn silent_until_started_and_after_stop() {
        let (mut control, mut rt) = player();
        assert_eq!(peak(&mut rt, 256), 0.0);
        control.start(schedule(), 0, false).expect("queue has room");
        assert!(control.is_playing());
        assert!(peak(&mut rt, 256) > 0.0);
        control.stop().expect("queue has room");
        assert_eq!(peak(&mut rt, 256), 0.0);
        assert!(!control.is_playing());
    }

    #[test]
    fn finishes_and_reports_position() {
        let (mut control, mut rt) = player();
        control.start(schedule(), 0, false).expect("queue has room");
        for _ in 0..40 {
            peak(&mut rt, 1_000);
        }
        // One bar at 120 bpm is 2 s = 16000 samples at 8 kHz, plus the release tail.
        assert!(!control.is_playing());
        assert!(control.position() >= 16_000);
    }

    #[test]
    fn looping_wraps_around() {
        let (mut control, mut rt) = player();
        control
            .start(schedule(), 15_900, true)
            .expect("queue has room");
        peak(&mut rt, 200);
        assert!(control.is_playing());
        assert!(
            control.position() < 1_000,
            "position {}",
            control.position()
        );
    }

    #[test]
    fn update_keeps_position() {
        let (mut control, mut rt) = player();
        control
            .start(schedule(), 4_000, false)
            .expect("queue has room");
        peak(&mut rt, 100);
        control.update(schedule()).expect("queue has room");
        peak(&mut rt, 100);
        assert_eq!(control.position(), 4_200);
        control.collect_garbage();
    }
}
