//! Older project files load into the current model, and are written back in the current
//! format.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use soundbit_core::{Filter, Instrument, PitchEnvelope, Reverb, ReverbSend, Unison, Vibrato};
use soundbit_formats::{FORMAT_VERSION, from_project_json, to_project_json};

const V1: &str = include_str!("fixtures/v1-two-voices.sbproj");

fn has_no_sound_shaping(instrument: &Instrument) -> bool {
    instrument.pitch_envelope == PitchEnvelope::default()
        && instrument.vibrato == Vibrato::default()
        && instrument.unison == Unison::default()
        && instrument.filter == Filter::default()
        && instrument.reverb_send == ReverbSend::default()
}

#[test]
fn version_1_loads_with_shaping_off() {
    let song = from_project_json(V1).expect("version 1 loads");
    assert_eq!(song.meta().title, "Two Voices");
    assert_eq!(song.instruments().count(), 2);
    assert!(song.instruments().all(|(_, i)| has_no_sound_shaping(i)));
    assert_eq!(song.reverb(), Reverb::default());
}

#[test]
fn version_1_is_saved_as_the_current_version() {
    let song = from_project_json(V1).expect("version 1 loads");
    let text = to_project_json(&song).expect("serializes");
    assert!(text.contains(&format!("\"format_version\": {FORMAT_VERSION}")));
    assert_eq!(from_project_json(&text).expect("reloads"), song);
}
