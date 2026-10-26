//! Pitches, pitch classes and note-name parsing.

use std::fmt;
use std::str::FromStr;

use crate::ModelError;
use crate::ranged::ranged;

ranged! {
    /// A MIDI note number. Middle C (`C4`) is 60 and `A4` (440 Hz) is 69.
    Pitch(u8): 0..=127, default 60, "pitch"
}

ranged! {
    /// A pitch class independent of octave: 0 is C, 1 is C#, ..., 11 is B.
    PitchClass(u8): 0..=11, default 0, "pitch class"
}

const SEMITONES_PER_OCTAVE: i32 = 12;
const SHARP_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

impl Pitch {
    /// Builds a pitch from a pitch class and an octave in scientific pitch notation.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::OutOfRange`] if the result is outside the MIDI range.
    pub fn from_class_and_octave(class: PitchClass, octave: i8) -> Result<Self, ModelError> {
        pitch_from_semitone(octave_base(octave) + i32::from(class.0))
    }

    /// Pitch class of this pitch.
    pub const fn class(self) -> PitchClass {
        PitchClass(self.0 % 12)
    }

    /// Octave in scientific pitch notation (`C4` is in octave 4).
    pub const fn octave(self) -> i8 {
        (self.0 / 12).cast_signed() - 1
    }

    /// Moves the pitch by a number of semitones.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::OutOfRange`] if the result is outside the MIDI range.
    pub fn transpose(self, semitones: i32) -> Result<Self, ModelError> {
        pitch_from_semitone(i32::from(self.0).saturating_add(semitones))
    }
}

/// MIDI number of the C that starts `octave`.
pub(crate) fn octave_base(octave: i8) -> i32 {
    (i32::from(octave) + 1) * SEMITONES_PER_OCTAVE
}

/// Converts an absolute semitone number to a pitch, checking the MIDI range.
pub(crate) fn pitch_from_semitone(semitone: i32) -> Result<Pitch, ModelError> {
    u8::try_from(semitone)
        .map_err(|_| ModelError::OutOfRange {
            name: "pitch",
            value: i64::from(semitone),
            min: i64::from(Pitch::MIN.0),
            max: i64::from(Pitch::MAX.0),
        })
        .and_then(Pitch::new)
}

/// Splits a leading note letter with an optional accidental from the rest of the string.
/// Returns the semitone offset from C, which may be -1 (`Cb`) or 12 (`B#`).
fn split_note_letter(text: &str) -> Option<(i32, &str)> {
    let mut chars = text.chars();
    let base = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest = chars.as_str();
    if let Some(rest) = rest.strip_prefix('#') {
        Some((base + 1, rest))
    } else if let Some(rest) = rest.strip_prefix('b') {
        Some((base - 1, rest))
    } else {
        Some((base, rest))
    }
}

impl FromStr for Pitch {
    type Err = ModelError;

    /// Parses names like `C4`, `f#3`, `Bb-1` or `B#3` (= `C4`).
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || ModelError::InvalidNoteName(text.to_owned());
        let (offset, octave) = split_note_letter(text.trim()).ok_or_else(invalid)?;
        let octave: i8 = octave.parse().map_err(|_| invalid())?;
        pitch_from_semitone(octave_base(octave) + offset)
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.class(), self.octave())
    }
}

impl FromStr for PitchClass {
    type Err = ModelError;

    /// Parses names like `C`, `F#` or `Bb`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || ModelError::InvalidNoteName(text.to_owned());
        let (offset, rest) = split_note_letter(text.trim()).ok_or_else(invalid)?;
        if !rest.is_empty() {
            return Err(invalid());
        }
        let class = u8::try_from(offset.rem_euclid(SEMITONES_PER_OCTAVE)).map_err(|_| invalid())?;
        Self::new(class)
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(SHARP_NAMES[usize::from(self.0)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pitch(name: &str) -> Pitch {
        name.parse().expect("valid note name")
    }

    #[test]
    fn parses_reference_pitches() {
        assert_eq!(pitch("C4").get(), 60);
        assert_eq!(pitch("A4").get(), 69);
        assert_eq!(pitch("C-1").get(), 0);
        assert_eq!(pitch("G9").get(), 127);
    }

    #[test]
    fn parses_accidentals_and_lowercase() {
        assert_eq!(pitch("f#3").get(), 54);
        assert_eq!(pitch("Bb2").get(), 46);
        assert_eq!(pitch("bb2").get(), 46);
        assert_eq!(pitch("B#3"), pitch("C4"));
        assert_eq!(pitch("Cb4"), pitch("B3"));
    }

    #[test]
    fn rejects_invalid_names() {
        for name in ["", "H4", "C", "C#", "4", "C4x", "C99999"] {
            assert!(name.parse::<Pitch>().is_err(), "{name} should be rejected");
        }
        assert!(matches!(
            "G#9".parse::<Pitch>(),
            Err(ModelError::OutOfRange { .. })
        ));
    }

    #[test]
    fn displays_with_sharps() {
        assert_eq!(pitch("Db4").to_string(), "C#4");
        assert_eq!(pitch("C-1").to_string(), "C-1");
    }

    #[test]
    fn parses_pitch_classes() {
        assert_eq!("Bb".parse::<PitchClass>().map(PitchClass::get), Ok(10));
        assert_eq!("B#".parse::<PitchClass>().map(PitchClass::get), Ok(0));
        assert_eq!("Cb".parse::<PitchClass>().map(PitchClass::get), Ok(11));
        assert!("C4".parse::<PitchClass>().is_err());
    }

    #[test]
    fn transposes_within_range() {
        assert_eq!(pitch("C4").transpose(12), Ok(pitch("C5")));
        assert!(pitch("C4").transpose(100).is_err());
    }
}
