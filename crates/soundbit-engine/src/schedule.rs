//! Songs compiled into sample-accurate playback plans.

use soundbit_core::{InstrumentId, Song, Track};
use soundbit_dsp::ReverbParams;

use crate::clock::StepClock;
use crate::error::{EngineError, check_sample_rate};
use crate::patch::Patch;

/// A note with absolute sample positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NoteEvent {
    /// Sample at which the note starts.
    pub(crate) on: u64,
    /// Sample at which the note is released.
    pub(crate) off: u64,
    /// Index into [`Schedule::patches`].
    pub(crate) patch: usize,
    /// MIDI pitch.
    pub(crate) pitch: u8,
    /// Velocity as a gain in `(0, 1]`.
    pub(crate) velocity: f32,
}

/// Everything one track plays, in start order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TrackPlan {
    pub(crate) gain: f32,
    pub(crate) events: Vec<NoteEvent>,
}

/// Converts between bar positions and sample positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionMap {
    clock: StepClock,
    steps_per_bar: u64,
}

impl PositionMap {
    /// Sample at which `bar` begins.
    pub fn bar_start(&self, bar: u16) -> u64 {
        self.clock.sample_at(u64::from(bar) * self.steps_per_bar)
    }

    /// Position of `sample` in bars (fractional), ignoring swing.
    pub fn bars_at(&self, sample: u64) -> f64 {
        self.clock.step_at(sample) as f64 / self.steps_per_bar as f64
    }
}

/// A song compiled for playback at one sample rate.
///
/// Instruments become DSP patches and notes become events at exact sample positions. Built off
/// the audio thread; the engine only reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    sample_rate: u32,
    patches: Vec<Patch>,
    tracks: Vec<TrackPlan>,
    end: u64,
    clock: StepClock,
    steps_per_bar: u64,
    loop_region: Option<(u64, u64)>,
    reverb: ReverbParams,
    uses_reverb: bool,
}

impl Schedule {
    /// Compiles `song` for playback at `sample_rate`.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidSampleRate`] for unsupported sample rates.
    pub fn compile(song: &Song, sample_rate: u32) -> Result<Self, EngineError> {
        check_sample_rate(sample_rate)?;
        let clock = StepClock::new(song.timing(), sample_rate);
        let steps_per_bar = u64::from(song.timing().steps_per_bar());

        let instrument_ids: Vec<InstrumentId> = song.instruments().map(|(id, _)| id).collect();
        let patches: Vec<Patch> = song
            .instruments()
            .map(|(_, instrument)| Patch::compile(instrument, sample_rate))
            .collect();
        let uses_reverb = patches.iter().any(|patch| patch.reverb_send > 0.0);
        let room = song.reverb();
        let reverb = ReverbParams {
            size: f32::from(room.size.get()) / 100.0,
            damping: f32::from(room.damping.get()) / 100.0,
        };
        let tracks = song
            .tracks()
            .map(|(_, track)| plan_track(song, track, &clock, &instrument_ids))
            .collect();
        let end = clock.sample_at(u64::from(song.length_bars()) * steps_per_bar);
        let loop_region = song.loop_range().map(|range| {
            let bar = |bar: u16| clock.sample_at(u64::from(bar) * steps_per_bar);
            (bar(range.start()), bar(range.end()))
        });

        Ok(Self {
            sample_rate,
            patches,
            tracks,
            end,
            clock,
            steps_per_bar,
            loop_region,
            reverb,
            uses_reverb,
        })
    }

    /// Converts between bars and samples for this schedule. Cheap to copy, so it can be kept
    /// on the control side after the schedule moves to the audio thread.
    pub fn position_map(&self) -> PositionMap {
        PositionMap {
            clock: self.clock,
            steps_per_bar: self.steps_per_bar.max(1),
        }
    }

    /// The song's loop region in samples, if it has one.
    pub fn loop_region(&self) -> Option<(u64, u64)> {
        self.loop_region
    }

    /// Sample rate the schedule was compiled for.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Sample at which the arrangement ends. Releasing notes may ring on after it.
    pub fn end_sample(&self) -> u64 {
        self.end
    }

    /// The song's reverb room.
    pub(crate) fn reverb(&self) -> ReverbParams {
        self.reverb
    }

    /// Whether any instrument sends to the reverb. Songs that don't skip it entirely.
    pub(crate) fn uses_reverb(&self) -> bool {
        self.uses_reverb
    }

    pub(crate) fn patches(&self) -> &[Patch] {
        &self.patches
    }

    pub(crate) fn tracks(&self) -> &[TrackPlan] {
        &self.tracks
    }
}

fn plan_track(
    song: &Song,
    track: &Track,
    clock: &StepClock,
    instrument_ids: &[InstrumentId],
) -> TrackPlan {
    let settings = track.settings();
    if settings.muted {
        return TrackPlan {
            gain: 0.0,
            events: Vec::new(),
        };
    }

    let steps_per_bar = u64::from(song.timing().steps_per_bar());
    let mut events = Vec::new();
    for (bar, pattern_id) in track.clips() {
        let Some(pattern) = song.pattern(pattern_id) else {
            continue;
        };
        let Some(patch) = instrument_ids
            .iter()
            .position(|&id| id == pattern.instrument)
        else {
            continue;
        };
        let bar_start = u64::from(bar) * steps_per_bar;
        for note in pattern.notes() {
            let start = bar_start + u64::from(note.start);
            events.push(NoteEvent {
                on: clock.sample_at(start),
                off: clock.sample_at(start + u64::from(note.length.get())),
                patch,
                pitch: note.pitch.get(),
                velocity: f32::from(note.velocity.get()) / 127.0,
            });
        }
    }
    // Clips come in bar order and notes in start order, but a stable sort keeps the invariant
    // explicit.
    events.sort_by_key(|event| event.on);
    TrackPlan {
        gain: f32::from(settings.volume.get()) / 100.0,
        events,
    }
}
