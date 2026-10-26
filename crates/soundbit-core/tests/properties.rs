//! Property tests for model invariants.

#![allow(
    clippy::expect_used,
    reason = "test helpers panic on invalid test data by design"
)]

use proptest::prelude::*;
use soundbit_core::{InstrumentId, Note, Pattern, Pitch};

#[test]
fn every_pitch_name_round_trips() {
    for value in 0..=127 {
        let pitch = Pitch::new(value).expect("in range");
        let parsed: Pitch = pitch.to_string().parse().expect("own names parse");
        assert_eq!(parsed, pitch);
    }
}

fn arb_note() -> impl Strategy<Value = Note> {
    (48u8..=72, 0u16..16, 1u16..=8).prop_map(|(pitch, start, length)| {
        Note::new(Pitch::new(pitch).expect("in range"), start, length).expect("non-zero length")
    })
}

fn assert_invariants(pattern: &Pattern) {
    let notes = pattern.notes();
    assert!(
        notes
            .windows(2)
            .all(|pair| (pair[0].start, pair[0].pitch) < (pair[1].start, pair[1].pitch)),
        "notes must be strictly sorted: {notes:?}"
    );
    for (i, a) in notes.iter().enumerate() {
        for b in &notes[i + 1..] {
            assert!(
                a.pitch != b.pitch || !a.overlaps_in_time(b),
                "same-pitch overlap: {a:?} {b:?}"
            );
        }
    }
}

proptest! {
    #[test]
    fn inserting_any_notes_keeps_pattern_valid(notes in prop::collection::vec(arb_note(), 0..40)) {
        let mut pattern = Pattern::new("p", InstrumentId::new(1));
        let mut accepted = 0;
        for note in notes {
            if pattern.insert_note(note).is_ok() {
                accepted += 1;
            }
        }
        assert_invariants(&pattern);
        prop_assert_eq!(pattern.notes().len(), accepted);
    }

    #[test]
    fn set_notes_matches_sorted_insertion(notes in prop::collection::vec(arb_note(), 0..40)) {
        let mut inserted = Pattern::new("p", InstrumentId::new(1));
        let all_fit = notes.iter().all(|note| inserted.insert_note(*note).is_ok());

        let mut replaced = Pattern::new("p", InstrumentId::new(1));
        let result = replaced.set_notes(notes);
        prop_assert_eq!(result.is_ok(), all_fit);
        if all_fit {
            prop_assert_eq!(replaced, inserted);
        }
    }

    #[test]
    fn pattern_json_round_trips(notes in prop::collection::vec(arb_note(), 0..40)) {
        let mut pattern = Pattern::new("p", InstrumentId::new(1));
        for note in notes {
            let _ = pattern.insert_note(note);
        }
        let json = serde_json::to_string(&pattern).expect("serializes");
        let restored: Pattern = serde_json::from_str(&json).expect("deserializes");
        prop_assert_eq!(restored, pattern);
    }

    #[test]
    fn polyphony_never_exceeds_note_count(notes in prop::collection::vec(arb_note(), 0..40)) {
        let mut pattern = Pattern::new("p", InstrumentId::new(1));
        for note in notes {
            let _ = pattern.insert_note(note);
        }
        prop_assert!(pattern.polyphony() <= pattern.notes().len());
        prop_assert_eq!(pattern.polyphony() == 0, pattern.notes().is_empty());
    }
}
