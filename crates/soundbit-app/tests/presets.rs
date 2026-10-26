//! Every built-in preset is valid, uniquely named and renders cleanly.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use std::collections::HashSet;

use soundbit_app::model::{Note, Pattern, Pitch, Song, TrackSettings};
use soundbit_app::{PresetLibrary, RenderOptions, Session};

#[test]
fn ids_are_unique_snake_case_and_documented() {
    let library = PresetLibrary::builtin().expect("built-in presets parse");
    assert!(
        library.all().len() >= 20,
        "only {} presets",
        library.all().len()
    );
    let mut seen = HashSet::new();
    for preset in library.all() {
        assert!(
            seen.insert(preset.id.as_str()),
            "duplicate id {}",
            preset.id
        );
        assert!(
            preset
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "{} is not snake_case",
            preset.id
        );
        assert!(
            !preset.description.is_empty(),
            "{} has no description",
            preset.id
        );
        assert!(!preset.source.is_empty(), "{} has no source", preset.id);
    }
    assert!(library.find("kick").is_some());
    assert!(library.find("no_such_preset").is_none());
}

/// A song playing `preset` as a C-major chord over one bar, at a register that suits it.
fn demo(preset: &soundbit_app::Preset) -> Song {
    let mut song = Song::new(&preset.id);
    let instrument = song
        .add_instrument(preset.instrument.clone())
        .expect("room");
    let root: u8 = match preset.category.as_str() {
        "drums" | "percussion" | "bass" => 36,
        _ => 60,
    };
    let mut pattern = Pattern::new("Demo", instrument);
    let notes = [0, 4, 7].map(|interval| {
        Note::new(Pitch::new(root + interval).expect("valid"), 0, 8).expect("valid")
    });
    pattern.set_notes(notes.to_vec()).expect("valid notes");
    let pattern = song.add_pattern(pattern).expect("valid");
    let track = song.add_track(TrackSettings::new("T")).expect("room");
    song.place_clip(track, 0, pattern).expect("valid");
    song
}

#[test]
fn every_preset_is_audible_and_unclipped() {
    let library = PresetLibrary::builtin().expect("built-in presets parse");
    let dir = tempfile::tempdir().expect("temp dir");
    for preset in library.all() {
        let session = Session::new(demo(preset));
        let options = RenderOptions {
            sample_rate: 22_050,
            ..RenderOptions::default()
        };
        let report = session
            .export_wav(dir.path().join("preset.wav"), &options)
            .expect("renders");
        assert!(
            report.peak > 0.02,
            "{} is nearly silent: {}",
            preset.id,
            report.peak
        );
        assert!(report.peak < 1.0, "{} clips", preset.id);
    }
}
