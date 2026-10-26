//! Sessions open, save and export real files.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use std::error::Error;
use std::fs;

use soundbit_app::model::{Instrument, Note, Pattern, Song, TrackSettings, Voice};
use soundbit_app::{AppError, RenderOptions, Session};

/// The error and all its causes, joined like `anyhow`'s `{:#}`.
fn chain(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(next) = cause {
        message = format!("{message}: {next}");
        cause = next.source();
    }
    message
}

fn one_note_song() -> Song {
    let mut song = Song::new("One note");
    let instrument = song
        .add_instrument(Instrument::new("Lead", Voice::default()))
        .expect("room");
    let mut pattern = Pattern::new("A", instrument);
    let pitch = "A4".parse().expect("valid pitch");
    pattern
        .insert_note(Note::new(pitch, 0, 4).expect("valid"))
        .expect("empty pattern");
    let pattern = song.add_pattern(pattern).expect("valid");
    let track = song.add_track(TrackSettings::new("Lead")).expect("room");
    song.place_clip(track, 0, pattern).expect("valid");
    song
}

#[test]
fn save_then_open_restores_the_song() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("song.sbproj");

    let mut session = Session::new(one_note_song());
    session.save_as(&path).expect("writable");
    assert_eq!(session.path(), Some(path.as_path()));

    let reopened = Session::open(&path).expect("valid project");
    assert_eq!(reopened.song(), session.song());
}

#[test]
fn export_writes_a_wav_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("song.wav");

    let report = Session::new(one_note_song())
        .export_wav(&path, &RenderOptions::default())
        .expect("exports");
    let bytes = fs::read(&path).expect("file exists");

    assert_eq!(&bytes[0..4], b"RIFF");
    // One bar at 120 bpm is two seconds, plus a short release.
    assert!(report.duration_seconds > 2.0 && report.duration_seconds < 2.5);
    assert!(report.peak > 0.0);
}

#[test]
fn errors_name_the_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let missing = dir.path().join("missing.sbproj");
    let error = Session::open(&missing).expect_err("file does not exist");
    assert!(matches!(error, AppError::Read { .. }));
    assert!(
        chain(&error).starts_with("cannot read "),
        "{}",
        chain(&error)
    );
    assert!(chain(&error).contains("missing.sbproj"));

    let invalid = dir.path().join("invalid.sbproj");
    fs::write(&invalid, "{}").expect("writable");
    let error = Session::open(&invalid).expect_err("not a project");
    assert!(matches!(error, AppError::Format { .. }));
    assert!(
        chain(&error).ends_with(
            "invalid.sbproj: not a SoundBit project: expected \"format\": \"soundbit-project\""
        ),
        "{}",
        chain(&error)
    );
}
