//! Per-track playback: triggering events and managing the voice pool.

use crate::patch::Patch;
use crate::schedule::TrackPlan;
use crate::voice::{Voice, VoiceFrame};

/// Notes a single track can sound at once. When all are busy, the oldest note is cut.
pub(crate) const VOICES_PER_TRACK: usize = 8;

/// Playback state of one track. Holds a fixed voice pool, so playing never allocates.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TrackPlayer {
    voices: [Voice; VOICES_PER_TRACK],
    next_event: usize,
}

impl TrackPlayer {
    pub(crate) fn new() -> Self {
        Self {
            voices: [Voice::default(); VOICES_PER_TRACK],
            next_event: 0,
        }
    }

    // RT-SAFE
    /// Moves to `position` in `plan`: the next note to start is the first one at or after it.
    /// Sounding notes keep playing.
    pub(crate) fn seek(&mut self, position: u64, plan: &TrackPlan) {
        self.next_event = plan.events.partition_point(|event| event.on < position);
    }

    // RT-SAFE
    /// Releases every sounding note at `now`, letting it ring out.
    pub(crate) fn release_all(&mut self, now: u64) {
        for voice in &mut self.voices {
            voice.release_by(now);
        }
    }

    // RT-SAFE
    /// Silences every voice immediately and forgets the event position.
    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    /// Whether every event has been played and every voice has finished.
    pub(crate) fn is_idle(&self, plan: &TrackPlan) -> bool {
        self.next_event >= plan.events.len() && self.voices.iter().all(|voice| !voice.is_active())
    }

    // RT-SAFE
    /// Starts any notes due at `now` and renders one frame: stereo output plus reverb send.
    pub(crate) fn render(
        &mut self,
        now: u64,
        plan: &TrackPlan,
        patches: &[Patch],
        sample_rate: f64,
    ) -> VoiceFrame {
        while let Some(event) = plan.events.get(self.next_event) {
            if event.on > now {
                break;
            }
            if let Some(patch) = patches.get(event.patch) {
                self.free_voice().start(patch, event, sample_rate);
            }
            self.next_event += 1;
        }

        let (mut left, mut right, mut send) = (0.0, 0.0, 0.0);
        for voice in &mut self.voices {
            let frame = voice.render(now);
            left += frame.left;
            right += frame.right;
            send += frame.send;
        }
        VoiceFrame {
            left: left * plan.gain,
            right: right * plan.gain,
            send: send * plan.gain,
        }
    }

    /// An idle voice, or the one that started first if all are busy.
    fn free_voice(&mut self) -> &mut Voice {
        let index = self
            .voices
            .iter()
            .position(|voice| !voice.is_active())
            .or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, voice)| voice.started_at())
                    .map(|(index, _)| index)
            })
            .unwrap_or(0);
        &mut self.voices[index]
    }
}
