//! Commands, undo/redo and the chip-profile policy.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use soundbit_app::command::{
    EnvelopeChanges, FilterChanges, InstrumentChanges, InstrumentCommand, MetaChanges, NoteKey,
    PatternChanges, PatternCommand, PitchEnvelopeChanges, ReverbChanges, SongCommand,
    TimingChanges, TrackChanges, TrackCommand, UnisonChanges, VibratoChanges,
};
use soundbit_app::model::{
    Bpm, ChipProfile, DutyCycle, EnvelopeTime, FilterCutoff, FilterMode, Instrument, InstrumentId,
    Level, LoopRange, ModelError, Note, Pattern, PatternId, Pitch, PitchSweep, ReverbSend,
    RoomSize, Scale, ScaleKind, Song, TrackId, TrackSettings, UnisonVoices, VibratoDepth, Voice,
    Volume,
};
use soundbit_app::{Command, CommandError, Created, Editor, Origin, ProfilePolicy, Session};

fn pitch(name: &str) -> Pitch {
    name.parse().expect("valid note name")
}

fn note(name: &str, start: u16, length: u16) -> Note {
    Note::new(pitch(name), start, length).expect("non-zero length")
}

fn run(editor: &mut Editor, command: impl Into<Command>) -> Vec<Created> {
    editor
        .execute(command, Origin::User)
        .expect("command succeeds")
        .created
}

fn add_instrument(editor: &mut Editor, voice: Voice) -> InstrumentId {
    match run(
        editor,
        InstrumentCommand::Add(Instrument::new("Inst", voice)),
    )[..]
    {
        [Created::Instrument(id)] => id,
        ref other => panic!("unexpected {other:?}"),
    }
}

fn create_pattern(editor: &mut Editor, instrument: InstrumentId, notes: &[Note]) -> PatternId {
    let mut pattern = Pattern::new("Pattern", instrument);
    pattern.set_notes(notes.to_vec()).expect("valid notes");
    match run(editor, PatternCommand::Create(pattern))[..] {
        [Created::Pattern(id)] => id,
        ref other => panic!("unexpected {other:?}"),
    }
}

fn add_track(editor: &mut Editor) -> TrackId {
    match run(editor, TrackCommand::Add(TrackSettings::new("Track")))[..] {
        [Created::Track(id)] => id,
        ref other => panic!("unexpected {other:?}"),
    }
}

/// An editor with a pulse instrument, a two-note pattern, and a track playing it on bar 0.
fn small_editor() -> (Editor, InstrumentId, PatternId, TrackId) {
    let mut editor = Editor::new(Song::new("Test"));
    let lead = add_instrument(
        &mut editor,
        Voice::Pulse {
            duty: DutyCycle::Half,
        },
    );
    let pattern = create_pattern(&mut editor, lead, &[note("C4", 0, 4), note("E4", 4, 4)]);
    let track = add_track(&mut editor);
    run(
        &mut editor,
        TrackCommand::PlaceClips {
            track,
            pattern,
            bars: vec![0],
        },
    );
    (editor, lead, pattern, track)
}

/// Every command kind, applied in sequence to the small editor's song.
/// Changes every sound-shaping group of an instrument at once.
fn shaping_changes() -> InstrumentChanges {
    InstrumentChanges {
        pitch_envelope: PitchEnvelopeChanges {
            semitones: Some(PitchSweep::new(24).expect("ok")),
            time: Some(EnvelopeTime::new(40).expect("ok")),
        },
        vibrato: VibratoChanges {
            depth: Some(VibratoDepth::new(30).expect("ok")),
            ..Default::default()
        },
        unison: UnisonChanges {
            voices: Some(UnisonVoices::new(3).expect("ok")),
            ..Default::default()
        },
        filter: FilterChanges {
            mode: Some(FilterMode::LowPass),
            cutoff: Some(FilterCutoff::new(800).expect("ok")),
            ..Default::default()
        },
        reverb_send: Some(ReverbSend::new(50).expect("ok")),
        ..Default::default()
    }
}

#[test]
fn shaping_changes_keep_unset_fields() {
    let (mut editor, lead, _, _) = small_editor();
    run(
        &mut editor,
        InstrumentCommand::Update(lead, shaping_changes()),
    );
    let deeper = InstrumentChanges {
        vibrato: VibratoChanges {
            depth: Some(VibratoDepth::new(60).expect("ok")),
            ..Default::default()
        },
        ..Default::default()
    };
    run(&mut editor, InstrumentCommand::Update(lead, deeper));
    let instrument = editor.song().instrument(lead).expect("exists");
    assert_eq!(instrument.vibrato.depth.get(), 60);
    assert_eq!(instrument.unison.voices.get(), 3);
    assert_eq!(instrument.filter.cutoff.get(), 800);
    assert_eq!(instrument.reverb_send.get(), 50);
}

/// Every song-wide command once.
fn song_commands() -> Vec<Command> {
    vec![
        SongCommand::UpdateMeta(MetaChanges {
            title: Some("New".into()),
            ..Default::default()
        })
        .into(),
        SongCommand::UpdateTiming(TimingChanges {
            bpm: Some(Bpm::new(90).expect("ok")),
            ..Default::default()
        })
        .into(),
        SongCommand::SetScale(Scale::new(
            "A".parse().expect("ok"),
            ScaleKind::NaturalMinor,
        ))
        .into(),
        SongCommand::SetChipProfile(ChipProfile::C64).into(),
        SongCommand::SetLoopRange(Some(LoopRange::new(0, 2).expect("ok"))).into(),
        SongCommand::UpdateReverb(ReverbChanges {
            size: Some(RoomSize::new(90).expect("ok")),
            ..Default::default()
        })
        .into(),
    ]
}

fn every_command(lead: InstrumentId, pattern: PatternId, track: TrackId) -> Vec<Command> {
    let mut commands = song_commands();
    commands.extend([
        InstrumentCommand::Update(lead, shaping_changes()).into(),
        InstrumentCommand::Update(
            lead,
            InstrumentChanges {
                volume: Some(Volume::new(50).expect("ok")),
                envelope: EnvelopeChanges {
                    sustain: Some(Level::new(40).expect("ok")),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .into(),
        PatternCommand::AddNotes(pattern, vec![note("G4", 8, 4)]).into(),
        PatternCommand::RemoveNotes(
            pattern,
            vec![NoteKey {
                start: 0,
                pitch: pitch("C4"),
            }],
        )
        .into(),
        PatternCommand::Transpose(pattern, 12).into(),
        PatternCommand::Update(
            pattern,
            PatternChanges {
                name: Some("Renamed".into()),
                ..Default::default()
            },
        )
        .into(),
        PatternCommand::Duplicate {
            source: pattern,
            name: None,
        }
        .into(),
        PatternCommand::SetNotes(pattern, vec![note("A4", 0, 16)]).into(),
        TrackCommand::Update(
            track,
            TrackChanges {
                muted: Some(true),
                ..Default::default()
            },
        )
        .into(),
        TrackCommand::PlaceClips {
            track,
            pattern,
            bars: vec![1, 2, 3],
        }
        .into(),
        TrackCommand::ClearClips {
            track,
            bars: vec![0, 2],
        }
        .into(),
        TrackCommand::Add(TrackSettings::new("Extra")).into(),
        InstrumentCommand::Add(Instrument::new("Spare", Voice::Sine)).into(),
        TrackCommand::Remove(track).into(),
        PatternCommand::Remove(pattern).into(),
    ]);
    commands
}

#[test]
fn every_command_undoes_and_redoes_exactly() {
    let (mut editor, lead, pattern, track) = small_editor();
    let mut states = vec![editor.song().clone()];
    for command in every_command(lead, pattern, track) {
        let label = command.label();
        editor
            .execute(command, Origin::User)
            .unwrap_or_else(|e| panic!("{label}: {e}"));
        states.push(editor.song().clone());
    }

    for expected in states.iter().rev().skip(1) {
        editor
            .undo()
            .expect("undo applies")
            .expect("something to undo");
        assert_eq!(editor.song(), expected);
    }
    for expected in states.iter().skip(1) {
        editor
            .redo()
            .expect("redo applies")
            .expect("something to redo");
        assert_eq!(editor.song(), expected);
    }
    assert_eq!(editor.redo().expect("nothing left"), None);
}

#[test]
fn redo_restores_the_same_ids() {
    let (mut editor, ..) = small_editor();
    let created = run(
        &mut editor,
        InstrumentCommand::Add(Instrument::new("Bass", Voice::Triangle)),
    );
    editor.undo().expect("undo");
    editor.redo().expect("redo");
    let [Created::Instrument(id)] = created[..] else {
        panic!("expected an instrument")
    };
    assert_eq!(
        editor.song().instrument(id).map(|i| i.name.as_str()),
        Some("Bass")
    );
}

#[test]
fn failed_batch_changes_nothing() {
    let (mut editor, lead, pattern, _) = small_editor();
    let before = editor.clone();
    let batch = Command::Batch {
        label: "Two steps".into(),
        commands: vec![
            PatternCommand::AddNotes(pattern, vec![note("B4", 8, 2)]).into(),
            InstrumentCommand::Remove(lead).into(), // fails: still used
        ],
    };
    let error = editor
        .execute(batch, Origin::User)
        .expect_err("second step fails");
    assert!(matches!(
        error,
        CommandError::Model(ModelError::InstrumentInUse { .. })
    ));
    assert_eq!(editor, before);
}

#[test]
fn errors_explain_what_is_wrong() {
    let (mut editor, _, pattern, _) = small_editor();
    let overlap = editor.execute(
        PatternCommand::AddNotes(pattern, vec![note("C4", 2, 1)]),
        Origin::Agent,
    );
    assert!(matches!(
        overlap,
        Err(CommandError::Model(ModelError::OverlappingNote { .. }))
    ));

    let missing = editor
        .execute(
            PatternCommand::RemoveNotes(
                pattern,
                vec![NoteKey {
                    start: 9,
                    pitch: pitch("C4"),
                }],
            ),
            Origin::Agent,
        )
        .expect_err("no such note");
    assert_eq!(
        missing.to_string(),
        "pattern 1 has no note C4 starting at step 9"
    );

    let too_high = editor.execute(PatternCommand::Transpose(pattern, 100), Origin::Agent);
    assert!(matches!(
        too_high,
        Err(CommandError::Model(ModelError::OutOfRange { .. }))
    ));
}

#[test]
fn enforce_policy_rejects_new_profile_violations() {
    let (mut editor, _, _, track) = small_editor();
    run(&mut editor, SongCommand::SetChipProfile(ChipProfile::Nes));
    let triangle = add_instrument(&mut editor, Voice::Triangle);
    let bass = create_pattern(&mut editor, triangle, &[note("C3", 0, 8)]);

    // Track 1 is NES "Pulse 1", which cannot play a triangle.
    let place = TrackCommand::PlaceClips {
        track,
        pattern: bass,
        bars: vec![1],
    };
    let error = editor
        .execute(place.clone(), Origin::Agent)
        .expect_err("violates NES");
    assert!(
        error.to_string().contains("cannot produce triangle voices"),
        "{error}"
    );

    editor.set_policy(ProfilePolicy::Report);
    let outcome = editor
        .execute(place, Origin::Agent)
        .expect("reported, not rejected");
    assert_eq!(outcome.violations.len(), 1);
}

#[test]
fn agent_changes_are_labelled() {
    let (mut editor, _, pattern, _) = small_editor();
    let outcome = editor
        .execute(PatternCommand::Transpose(pattern, -2), Origin::Agent)
        .expect("valid");
    assert_eq!(outcome.label, "AI: Transpose pattern 1 by -2 semitones");
    assert_eq!(
        editor.undo_label(),
        Some("AI: Transpose pattern 1 by -2 semitones")
    );
}

#[test]
fn revision_tracks_every_change() {
    let (mut editor, _, pattern, _) = small_editor();
    let start = editor.revision();
    run(&mut editor, PatternCommand::Transpose(pattern, 1));
    editor.undo().expect("undo");
    editor.redo().expect("redo");
    assert_eq!(editor.revision(), start + 3);
}

#[test]
fn session_knows_when_it_is_modified() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut session = Session::new(Song::new("New"));
    assert!(!session.is_modified());
    session
        .editor_mut()
        .execute(TrackCommand::Add(TrackSettings::new("T")), Origin::User)
        .expect("valid");
    assert!(session.is_modified());
    session
        .save_as(dir.path().join("song.sbproj"))
        .expect("writable");
    assert!(!session.is_modified());
    session.editor_mut().undo().expect("undo");
    assert!(session.is_modified());
}
