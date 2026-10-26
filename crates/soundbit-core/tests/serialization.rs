//! Songs survive a JSON round trip, and invalid JSON is rejected with the model's own rules.

#![allow(
    clippy::expect_used,
    reason = "test helpers panic on invalid test data by design"
)]

mod common;

use common::demo_song;
use serde_json::{Value, json};
use soundbit_core::{FmVoice, Instrument, Song, Voice, Wavetable};

fn to_json(song: &Song) -> Value {
    serde_json::to_value(song).expect("songs always serialize")
}

fn from_json(value: Value) -> Result<Song, String> {
    serde_json::from_value(value).map_err(|error| error.to_string())
}

#[test]
fn round_trip_preserves_the_song() {
    let mut song = demo_song().song;
    song.add_instrument(Instrument::new(
        "Wave",
        Voice::Wavetable(Wavetable::triangle()),
    ))
    .expect("room");
    song.add_instrument(Instrument::new("Bell", Voice::Fm(FmVoice::default())))
        .expect("room");

    let text = serde_json::to_string_pretty(&song).expect("serializes");
    let restored: Song = serde_json::from_str(&text).expect("deserializes");
    assert_eq!(restored, song);
}

#[test]
fn layout_is_readable() {
    let json = to_json(&demo_song().song);
    assert_eq!(json["meta"]["title"], "Demo");
    assert_eq!(json["instruments"][0]["id"], 1);
    assert_eq!(
        json["instruments"][0]["voice"],
        json!({"type": "pulse", "duty": "quarter"})
    );
    assert_eq!(json["patterns"][0]["notes"][0]["pitch"], 72);
    assert_eq!(
        json["tracks"][0]["clips"],
        json!([{"bar": 1, "pattern": 1}, {"bar": 3, "pattern": 1}])
    );
}

#[test]
fn empty_object_is_an_empty_song() {
    let song = from_json(json!({})).expect("all fields have defaults");
    assert_eq!(song.track_count(), 0);
    assert_eq!(song.timing().steps_per_bar(), 16);
}

#[test]
fn ids_continue_after_loaded_ones() {
    let mut song = from_json(to_json(&demo_song().song)).expect("valid");
    let id = song
        .add_instrument(Instrument::new("New", Voice::Sine))
        .expect("room");
    assert_eq!(id.get(), 3);
}

/// Applies `edit` to the demo song's JSON and returns the deserialization error.
fn rejection(edit: impl FnOnce(&mut Value)) -> String {
    let mut json = to_json(&demo_song().song);
    edit(&mut json);
    from_json(json).expect_err("edited JSON must be rejected")
}

#[test]
fn rejects_out_of_range_values() {
    let error = rejection(|json| json["instruments"][0]["volume"] = json!(101));
    assert!(
        error.contains("volume must be between 0 and 100"),
        "{error}"
    );
}

#[test]
fn rejects_dangling_references() {
    let error = rejection(|json| json["patterns"][0]["instrument"] = json!(99));
    assert!(error.contains("instrument 99 does not exist"), "{error}");

    let error = rejection(|json| json["tracks"][0]["clips"][0]["pattern"] = json!(99));
    assert!(error.contains("pattern 99 does not exist"), "{error}");
}

#[test]
fn rejects_duplicate_ids() {
    let error = rejection(|json| json["instruments"][1]["id"] = json!(1));
    assert!(error.contains("duplicate instrument 1"), "{error}");
}

#[test]
fn rejects_overlapping_notes() {
    let error = rejection(|json| {
        let first = json["patterns"][0]["notes"][0].clone();
        json["patterns"][0]["notes"]
            .as_array_mut()
            .expect("array")
            .push(first);
    });
    assert!(error.contains("overlaps"), "{error}");
}

#[test]
fn rejects_two_clips_in_one_bar() {
    let error = rejection(|json| {
        let clips = json["tracks"][0]["clips"].as_array_mut().expect("array");
        clips.push(json!({"bar": 1, "pattern": 2}));
    });
    assert!(
        error.contains("two clips on the same track at bar 1"),
        "{error}"
    );
}

#[test]
fn rejects_notes_outside_the_bar() {
    let error = rejection(|json| json["patterns"][0]["notes"][0]["start"] = json!(15));
    assert!(
        error.contains("does not fit in a bar of 16 steps"),
        "{error}"
    );
}
