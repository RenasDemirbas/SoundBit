//! Notes written by name.

use schemars::JsonSchema;
use serde::Deserialize;
use soundbit_app::command::NoteKey;
use soundbit_app::model::{Note, Pitch, Velocity};

/// A note to write into a pattern.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub(crate) struct NoteInput {
    /// Note name with octave: "C4" is middle C; also "F#3", "Bb5".
    pub(crate) note: String,
    /// First step of the note, counted from 0.
    pub(crate) start: u16,
    /// Duration in steps. Defaults to 1.
    #[serde(default = "one_step")]
    pub(crate) length: u16,
    /// MIDI velocity 1-127. Defaults to 100.
    pub(crate) velocity: Option<u8>,
}

fn one_step() -> u16 {
    1
}

impl NoteInput {
    fn to_note(&self) -> Result<Note, String> {
        let pitch: Pitch = self.note.parse().map_err(|error| format!("{error}"))?;
        let mut note =
            Note::new(pitch, self.start, self.length).map_err(|error| error.to_string())?;
        if let Some(velocity) = self.velocity {
            note = note.with_velocity(Velocity::new(velocity).map_err(|error| error.to_string())?);
        }
        Ok(note)
    }
}

/// Converts notes, naming the position of the first invalid one.
pub(crate) fn notes(inputs: &[NoteInput]) -> Result<Vec<Note>, String> {
    inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            input
                .to_note()
                .map_err(|error| format!("notes[{index}]: {error}"))
        })
        .collect()
}

/// Identifies an existing note.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub(crate) struct NoteRef {
    /// Note name with octave, e.g. "C4".
    pub(crate) note: String,
    /// Start step of the note.
    pub(crate) start: u16,
}

impl NoteRef {
    pub(crate) fn to_key(&self) -> Result<NoteKey, String> {
        let pitch = self
            .note
            .parse()
            .map_err(|error: soundbit_app::model::ModelError| error.to_string())?;
        Ok(NoteKey {
            start: self.start,
            pitch,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(note: &str, start: u16, length: u16, velocity: Option<u8>) -> NoteInput {
        NoteInput {
            note: note.to_owned(),
            start,
            length,
            velocity,
        }
    }

    #[test]
    fn converts_valid_notes() {
        let converted = notes(&[input("A4", 2, 3, Some(90))]).expect("valid");
        assert_eq!(converted[0].pitch.get(), 69);
        assert_eq!((converted[0].start, converted[0].length.get()), (2, 3));
        assert_eq!(converted[0].velocity.get(), 90);
    }

    #[test]
    fn errors_name_the_note() {
        let error = notes(&[input("C4", 0, 1, None), input("H2", 0, 1, None)]).expect_err("bad");
        assert!(
            error.starts_with("notes[1]: invalid note name `H2`"),
            "{error}"
        );
        let error = notes(&[input("C4", 0, 0, None)]).expect_err("zero length");
        assert!(error.contains("at least 1 step"), "{error}");
    }

    #[test]
    fn length_defaults_to_one_step() {
        let note: NoteInput = serde_json::from_str(r#"{"note": "C4", "start": 0}"#).expect("valid");
        assert_eq!(note.length, 1);
    }
}
