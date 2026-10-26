//! Editing a song through the aggregate keeps it consistent.

mod common;

use common::{demo_song, note, pattern};
use soundbit_core::limits::{MAX_BARS, MAX_TRACKS};
use soundbit_core::{
    InstrumentId, ModelError, PatternId, StepsPerBeat, TimeSettings, TrackId, TrackSettings,
};

#[test]
fn demo_song_has_expected_shape() {
    let demo = demo_song();
    assert_eq!(demo.song.instruments().count(), 2);
    assert_eq!(demo.song.patterns().count(), 2);
    assert_eq!(demo.song.track_count(), 2);
    assert_eq!(demo.song.length_bars(), 4);
    // Four bars of 16 steps at 120 bpm with 4 steps per beat: 8 seconds.
    assert!((demo.song.length_seconds() - 8.0).abs() < 1e-9);
    let lead = demo.song.track(demo.lead_track).expect("track exists");
    assert_eq!(lead.clip_at(1), Some(demo.melody));
    assert_eq!(lead.clip_at(0), None);
}

#[test]
fn pattern_must_reference_existing_instrument() {
    let mut song = demo_song().song;
    let missing = InstrumentId::new(99);
    assert_eq!(
        song.add_pattern(pattern("Ghost", missing, &[])),
        Err(ModelError::UnknownInstrument(missing))
    );
}

#[test]
fn notes_must_fit_in_the_bar() {
    let demo = demo_song();
    let mut song = demo.song;
    let result = song.add_pattern(pattern("Long", demo.lead, &[note("C4", 14, 4)]));
    assert_eq!(
        result,
        Err(ModelError::NoteOutsideBar {
            start: 14,
            length: 4,
            steps_per_bar: 16
        })
    );
}

#[test]
fn replace_pattern_returns_previous_version() {
    let demo = demo_song();
    let mut song = demo.song;
    let mut edited = song.pattern(demo.melody).expect("exists").clone();
    edited.insert_note(note("B5", 14, 2)).expect("no overlap");

    let previous = song
        .replace_pattern(demo.melody, edited.clone())
        .expect("valid");
    assert_eq!(previous.notes().len(), 3);
    assert_eq!(song.pattern(demo.melody), Some(&edited));
}

#[test]
fn rejected_edit_leaves_song_unchanged() {
    let demo = demo_song();
    let mut song = demo.song.clone();
    let invalid = pattern("Melody", demo.lead, &[note("C4", 15, 2)]);
    assert!(song.replace_pattern(demo.melody, invalid).is_err());
    assert_eq!(song, demo.song);
}

#[test]
fn cannot_remove_what_is_still_used() {
    let demo = demo_song();
    let mut song = demo.song;
    assert_eq!(
        song.remove_instrument(demo.bass),
        Err(ModelError::InstrumentInUse {
            instrument: demo.bass,
            pattern: demo.groove
        })
    );
    assert_eq!(
        song.remove_pattern(demo.melody),
        Err(ModelError::PatternInUse {
            pattern: demo.melody,
            track: demo.lead_track,
            bar: 1
        })
    );
}

#[test]
fn remove_then_restore_is_an_exact_undo() {
    let demo = demo_song();
    let original = demo.song.clone();
    let mut song = demo.song;

    let track = song.remove_track(demo.lead_track).expect("exists");
    let pattern = song.remove_pattern(demo.melody).expect("no longer used");
    let instrument = song.remove_instrument(demo.lead).expect("no longer used");

    song.restore_instrument(instrument).expect("id free");
    song.restore_pattern(pattern).expect("instrument back");
    song.restore_track(track).expect("pattern back");
    assert_eq!(song, original);
}

#[test]
fn restore_checks_references_again() {
    let demo = demo_song();
    let mut song = demo.song;
    let track = song.remove_track(demo.lead_track).expect("exists");
    song.remove_pattern(demo.melody).expect("no longer used");
    assert_eq!(
        song.restore_track(track),
        Err(ModelError::UnknownPattern(demo.melody))
    );
}

#[test]
fn shrinking_the_bar_requires_patterns_to_fit() {
    let mut song = demo_song().song;
    let three_quarter = TimeSettings {
        steps_per_beat: StepsPerBeat::new(3).expect("in range"),
        ..TimeSettings::default()
    };
    // The demo's notes end exactly at step 12, so a 12-step bar still fits.
    assert!(song.set_timing(three_quarter).is_ok());
    assert_eq!(song.timing().steps_per_bar(), 12);

    let tiny = TimeSettings {
        steps_per_beat: StepsPerBeat::new(2).expect("in range"),
        ..TimeSettings::default()
    };
    assert!(matches!(
        song.set_timing(tiny),
        Err(ModelError::PatternDoesNotFit {
            steps_per_bar: 8,
            ..
        })
    ));
    assert_eq!(song.timing().steps_per_bar(), 12);
}

#[test]
fn clips_validate_ids_and_bars() {
    let demo = demo_song();
    let mut song = demo.song;
    let missing_track = TrackId::new(42);
    let missing_pattern = PatternId::new(42);
    assert_eq!(
        song.place_clip(missing_track, 0, demo.melody),
        Err(ModelError::UnknownTrack(missing_track))
    );
    assert_eq!(
        song.place_clip(demo.lead_track, 0, missing_pattern),
        Err(ModelError::UnknownPattern(missing_pattern))
    );
    assert_eq!(
        song.place_clip(demo.lead_track, MAX_BARS, demo.melody),
        Err(ModelError::BarOutOfRange {
            bar: MAX_BARS,
            max: MAX_BARS
        })
    );
    assert_eq!(
        song.place_clip(demo.lead_track, 3, demo.groove),
        Ok(Some(demo.melody))
    );
    assert_eq!(song.clear_clip(demo.lead_track, 3), Ok(Some(demo.groove)));
    assert_eq!(song.clear_clip(demo.lead_track, 3), Ok(None));
}

#[test]
fn track_count_is_limited() {
    let mut song = demo_song().song;
    while song.track_count() < MAX_TRACKS {
        song.add_track(TrackSettings::new("More"))
            .expect("room left");
    }
    assert_eq!(
        song.add_track(TrackSettings::new("Too many")),
        Err(ModelError::LimitReached {
            what: "tracks",
            max: MAX_TRACKS
        })
    );
}
