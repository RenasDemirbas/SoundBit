//! Offline rendering: determinism, timing and levels.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

mod common;

use common::{audio_hash, shaped_song, showcase_song};
use soundbit_core::{Song, TrackSettings};
use soundbit_engine::{EngineError, RenderOptions, Rendered, Schedule, render};

/// Hash of the showcase song rendered at 48 kHz.
///
/// This pins the exact audio output. If a change to the DSP or engine alters the sound on
/// purpose, update the value and explain why in the pull request. If it changes by accident,
/// or differs between platforms, something broke determinism.
const SHOWCASE_HASH: u64 = 0x4cdc_1659_08bf_c6c1;

/// Hash of the sound-shaping song rendered at 48 kHz: pins pitch envelopes, vibrato,
/// unison, filters and reverb the same way [`SHOWCASE_HASH`] pins the bare voices.
const SHAPED_HASH: u64 = 0xcec6_770b_254c_a081;

fn render_showcase() -> Rendered {
    render(&showcase_song(), &RenderOptions::default()).expect("valid options")
}

#[test]
fn showcase_matches_golden_hash() {
    let hash = audio_hash(&render_showcase().samples);
    assert_eq!(
        hash, SHOWCASE_HASH,
        "golden audio changed: new hash is {hash:#018x}"
    );
}

#[test]
fn shaped_song_matches_golden_hash() {
    let rendered = render(&shaped_song(), &RenderOptions::default()).expect("valid options");
    let hash = audio_hash(&rendered.samples);
    assert_eq!(
        hash, SHAPED_HASH,
        "golden audio changed: new hash is {hash:#018x}"
    );
}

#[test]
fn shaped_song_is_deterministic_finite_and_unclipped() {
    let song = shaped_song();
    let first = render(&song, &RenderOptions::default()).expect("valid");
    assert_eq!(
        first,
        render(&song, &RenderOptions::default()).expect("valid")
    );
    assert!(first.samples.iter().all(|s| s.is_finite()));
    assert!(
        first.peak() > 0.1 && first.peak() < 1.0,
        "peak {}",
        first.peak()
    );
}

#[test]
fn reverb_tail_outlasts_the_dry_release() {
    let mut song = shaped_song();
    let wet = render(&song, &RenderOptions::default()).expect("valid");
    let ids: Vec<_> = song.instruments().map(|(id, _)| id).collect();
    for id in ids {
        let mut instrument = song.instrument(id).expect("exists").clone();
        instrument.reverb_send = soundbit_core::ReverbSend::MIN;
        song.replace_instrument(id, instrument).expect("exists");
    }
    let dry = render(&song, &RenderOptions::default()).expect("valid");
    assert!(
        wet.duration_seconds() > dry.duration_seconds() + 0.5,
        "wet {} s, dry {} s",
        wet.duration_seconds(),
        dry.duration_seconds()
    );
}

#[test]
fn reports_level_per_track() {
    let rendered = render(&shaped_song(), &RenderOptions::default()).expect("valid");
    assert_eq!(rendered.tracks.len(), 4);
    for track in &rendered.tracks {
        assert!(track.peak > 0.0 && track.mean_square > 0.0, "{track:?}");
    }
    assert!(rendered.mean_square() > 0.0);
}

#[test]
fn rendering_is_deterministic() {
    assert_eq!(render_showcase(), render_showcase());
}

#[test]
fn output_is_finite_bounded_and_audible() {
    let rendered = render_showcase();
    assert!(
        rendered
            .samples
            .iter()
            .all(|s| s.is_finite() && (-1.0..=1.0).contains(s))
    );
    assert!(rendered.peak() > 0.1, "peak {}", rendered.peak());
}

#[test]
fn length_covers_arrangement_plus_release() {
    let rendered = render_showcase();
    let song = showcase_song();
    let schedule = Schedule::compile(&song, 48_000).expect("valid rate");
    // Two bars of 16 steps at 150 bpm: 4800 samples per step.
    assert_eq!(schedule.end_sample(), 2 * 16 * 4_800);
    let frames = u64::try_from(rendered.frames()).expect("fits");
    assert!(frames >= schedule.end_sample());
    assert!(
        frames < schedule.end_sample() + 48_000,
        "tail too long: {frames}"
    );
}

#[test]
fn sample_rate_scales_length() {
    let song = showcase_song();
    let at = |sample_rate| {
        render(
            &song,
            &RenderOptions {
                sample_rate,
                ..RenderOptions::default()
            },
        )
        .expect("valid rate")
        .duration_seconds()
    };
    assert!((at(44_100) - at(96_000)).abs() < 0.01);
}

#[test]
fn empty_song_renders_nothing() {
    let rendered = render(&Song::new("Empty"), &RenderOptions::default()).expect("valid");
    assert_eq!(rendered.samples, []);
}

#[test]
fn muted_tracks_are_silent() {
    let mut song = showcase_song();
    let ids: Vec<_> = song.tracks().map(|(id, _)| id).collect();
    for id in ids {
        let mut settings = TrackSettings::new("muted");
        settings.muted = true;
        song.set_track_settings(id, settings).expect("track exists");
    }
    let rendered = render(&song, &RenderOptions::default()).expect("valid");
    assert_eq!(rendered.peak(), 0.0);
}

#[test]
fn rejects_unsupported_sample_rates() {
    let options = RenderOptions {
        sample_rate: 1_000,
        ..RenderOptions::default()
    };
    assert!(matches!(
        render(&showcase_song(), &options),
        Err(EngineError::InvalidSampleRate { got: 1_000, .. })
    ));
}
