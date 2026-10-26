//! Songs are checked against the limits of their chip profile.

mod common;

use common::{demo_song, note, pattern};
use soundbit_core::{ChipProfile, ProfileViolation, TrackSettings, VoiceKind};

#[test]
fn free_profile_never_complains() {
    let mut song = demo_song().song;
    song.set_chip_profile(ChipProfile::Free);
    assert_eq!(song.profile_violations(), []);
}

#[test]
fn every_constrained_profile_has_channels() {
    for profile in ChipProfile::ALL {
        if let Some(channels) = profile.channels() {
            assert!(!channels.is_empty(), "{profile:?}");
            assert!(
                channels
                    .iter()
                    .all(|c| !c.voices.is_empty() && c.polyphony > 0),
                "{profile:?}"
            );
        }
    }
}

#[test]
fn voices_must_match_their_channel() {
    // Demo: track 1 plays a pulse (fine on NES "Pulse 1"), track 2 plays a triangle bass,
    // which NES "Pulse 2" cannot produce.
    let demo = demo_song();
    let mut song = demo.song;
    song.set_chip_profile(ChipProfile::Nes);
    assert_eq!(
        song.profile_violations(),
        [ProfileViolation::VoiceNotAllowed {
            track: demo.bass_track,
            channel: "Pulse 2",
            bar: 0,
            pattern: demo.groove,
            voice: VoiceKind::Triangle,
        }]
    );

    song.set_chip_profile(ChipProfile::C64);
    assert_eq!(song.profile_violations(), []);
}

#[test]
fn channels_are_monophonic() {
    let demo = demo_song();
    let mut song = demo.song;
    song.set_chip_profile(ChipProfile::C64);
    let chord = pattern(
        "Chord",
        demo.lead,
        &[note("C4", 0, 4), note("E4", 0, 4), note("G4", 2, 4)],
    );
    let chord = song.add_pattern(chord).expect("valid pattern");
    song.place_clip(demo.lead_track, 0, chord)
        .expect("valid clip");

    let message = song.profile_violations()[0].to_string();
    assert_eq!(
        message,
        "track 1 plays on the Voice 1 channel, which plays 1 note(s) at a time, \
         but pattern 3 at bar 0 plays 3"
    );
}

#[test]
fn track_count_is_limited_by_channels() {
    let mut song = demo_song().song;
    for name in ["Three", "Four"] {
        song.add_track(TrackSettings::new(name)).expect("room");
    }
    song.set_chip_profile(ChipProfile::C64);
    assert!(
        song.profile_violations()
            .contains(&ProfileViolation::TooManyTracks {
                profile: "Commodore 64",
                tracks: 4,
                max: 3,
            })
    );
}

#[test]
fn repeated_clips_are_reported_once() {
    let demo = demo_song();
    let mut song = demo.song;
    song.set_chip_profile(ChipProfile::Nes);
    // The groove is placed on four bars but should produce a single violation.
    assert_eq!(song.profile_violations().len(), 1);
}
