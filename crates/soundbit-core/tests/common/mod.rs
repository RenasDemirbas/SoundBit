//! Shared fixtures for integration tests.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of the helpers"
)]
#![allow(
    clippy::expect_used,
    reason = "fixtures panic on invalid test data by design"
)]

use soundbit_core::{
    DutyCycle, Instrument, InstrumentId, Note, Pattern, PatternId, Pitch, Song, TrackId,
    TrackSettings, Voice,
};

/// Ids of the entities in [`demo_song`].
pub(crate) struct Demo {
    pub(crate) song: Song,
    pub(crate) lead: InstrumentId,
    pub(crate) bass: InstrumentId,
    pub(crate) melody: PatternId,
    pub(crate) groove: PatternId,
    pub(crate) lead_track: TrackId,
    pub(crate) bass_track: TrackId,
}

pub(crate) fn pitch(name: &str) -> Pitch {
    name.parse().expect("valid note name")
}

pub(crate) fn note(name: &str, start: u16, length: u16) -> Note {
    Note::new(pitch(name), start, length).expect("non-zero length")
}

pub(crate) fn pattern(name: &str, instrument: InstrumentId, notes: &[Note]) -> Pattern {
    let mut pattern = Pattern::new(name, instrument);
    pattern.set_notes(notes.to_vec()).expect("valid notes");
    pattern
}

/// A small two-track song: a pulse lead and a triangle bass over four bars.
pub(crate) fn demo_song() -> Demo {
    let mut song = Song::new("Demo");
    let lead = song
        .add_instrument(Instrument::new(
            "Lead",
            Voice::Pulse {
                duty: DutyCycle::Quarter,
            },
        ))
        .expect("room for instrument");
    let bass = song
        .add_instrument(Instrument::new("Bass", Voice::Triangle))
        .expect("room for instrument");

    let melody = song
        .add_pattern(pattern(
            "Melody",
            lead,
            &[note("C5", 0, 2), note("E5", 4, 2), note("G5", 8, 4)],
        ))
        .expect("valid pattern");
    let groove = song
        .add_pattern(pattern(
            "Groove",
            bass,
            &[note("C3", 0, 4), note("G2", 8, 4)],
        ))
        .expect("valid pattern");

    let lead_track = song
        .add_track(TrackSettings::new("Lead"))
        .expect("room for track");
    let bass_track = song
        .add_track(TrackSettings::new("Bass"))
        .expect("room for track");
    for bar in 0..4 {
        song.place_clip(bass_track, bar, groove)
            .expect("valid clip");
    }
    song.place_clip(lead_track, 1, melody).expect("valid clip");
    song.place_clip(lead_track, 3, melody).expect("valid clip");

    Demo {
        song,
        lead,
        bass,
        melody,
        groove,
        lead_track,
        bass_track,
    }
}
