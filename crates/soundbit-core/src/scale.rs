//! Musical scales, used to guide note entry in the editor and by AI agents.
//!
//! Scale interval tables are standard music theory, so they are domain constants rather than
//! data files.

use serde::{Deserialize, Serialize};

use crate::pitch::{octave_base, pitch_from_semitone};
use crate::{ModelError, Pitch, PitchClass};

/// The interval pattern of a scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ScaleKind {
    /// All twelve semitones; no guidance.
    #[default]
    Chromatic,
    /// Ionian mode.
    Major,
    /// Aeolian mode.
    NaturalMinor,
    /// Natural minor with a raised seventh.
    HarmonicMinor,
    /// Ascending melodic minor.
    MelodicMinor,
    /// Dorian mode.
    Dorian,
    /// Phrygian mode.
    Phrygian,
    /// Lydian mode.
    Lydian,
    /// Mixolydian mode.
    Mixolydian,
    /// Locrian mode.
    Locrian,
    /// Five-note major pentatonic.
    MajorPentatonic,
    /// Five-note minor pentatonic.
    MinorPentatonic,
    /// Minor pentatonic with the flat fifth "blue note".
    Blues,
    /// Six whole-tone steps.
    WholeTone,
}

impl ScaleKind {
    /// Every scale kind, in display order.
    pub const ALL: [Self; 14] = [
        Self::Chromatic,
        Self::Major,
        Self::NaturalMinor,
        Self::HarmonicMinor,
        Self::MelodicMinor,
        Self::Dorian,
        Self::Phrygian,
        Self::Lydian,
        Self::Mixolydian,
        Self::Locrian,
        Self::MajorPentatonic,
        Self::MinorPentatonic,
        Self::Blues,
        Self::WholeTone,
    ];

    /// Semitone offsets of each degree from the root, ascending, starting at 0.
    pub const fn intervals(self) -> &'static [u8] {
        match self {
            Self::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Self::Major => &[0, 2, 4, 5, 7, 9, 11],
            Self::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Self::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Self::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            Self::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Self::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Self::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Self::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Self::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Self::MajorPentatonic => &[0, 2, 4, 7, 9],
            Self::MinorPentatonic => &[0, 3, 5, 7, 10],
            Self::Blues => &[0, 3, 5, 6, 7, 10],
            Self::WholeTone => &[0, 2, 4, 6, 8, 10],
        }
    }
}

/// A key: a root pitch class plus a scale kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Scale {
    /// Root (tonic) of the scale.
    pub root: PitchClass,
    /// Interval pattern.
    pub kind: ScaleKind,
}

/// How far [`Scale::snap`] searches for an in-scale pitch. No scale has a gap wider than this.
const MAX_SNAP_DISTANCE: i32 = 6;

impl Scale {
    /// Creates a scale.
    pub const fn new(root: PitchClass, kind: ScaleKind) -> Self {
        Self { root, kind }
    }

    /// Whether `pitch` belongs to this scale.
    pub fn contains(self, pitch: Pitch) -> bool {
        let relative = (pitch.class().get() + 12 - self.root.get()) % 12;
        self.kind.intervals().contains(&relative)
    }

    /// The pitch of a scale degree. Degree 0 is the root in `octave`; degrees past the end of
    /// the scale continue into higher octaves and negative degrees go below.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::OutOfRange`] if the result is outside the MIDI range.
    pub fn pitch_at(self, degree: i32, octave: i8) -> Result<Pitch, ModelError> {
        let intervals = self.kind.intervals();
        let len = i32::from(u8::try_from(intervals.len()).unwrap_or(u8::MAX));
        let octave_shift = degree.div_euclid(len);
        let interval = usize::try_from(degree.rem_euclid(len))
            .ok()
            .and_then(|index| intervals.get(index))
            .copied()
            .unwrap_or(0);
        pitch_from_semitone(
            octave_base(octave)
                + octave_shift * 12
                + i32::from(self.root.get())
                + i32::from(interval),
        )
    }

    /// The nearest pitch in the scale, preferring the lower one on a tie.
    pub fn snap(self, pitch: Pitch) -> Pitch {
        (0..=MAX_SNAP_DISTANCE)
            .flat_map(|distance| [-distance, distance])
            .filter_map(|offset| pitch.transpose(offset).ok())
            .find(|candidate| self.contains(*candidate))
            .unwrap_or(pitch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c_major() -> Scale {
        Scale::new(PitchClass::default(), ScaleKind::Major)
    }

    fn pitch(name: &str) -> Pitch {
        name.parse().expect("valid note name")
    }

    #[test]
    fn every_scale_starts_on_the_root_and_ascends() {
        for kind in ScaleKind::ALL {
            let intervals = kind.intervals();
            assert_eq!(intervals.first(), Some(&0), "{kind:?}");
            assert!(
                intervals.windows(2).all(|pair| pair[0] < pair[1]),
                "{kind:?}"
            );
            assert!(intervals.iter().all(|&step| step < 12), "{kind:?}");
        }
    }

    #[test]
    fn contains_respects_root() {
        let a_minor = Scale::new(PitchClass::saturating(9), ScaleKind::NaturalMinor);
        assert!(a_minor.contains(pitch("C4")));
        assert!(!a_minor.contains(pitch("C#4")));
        assert!(c_major().contains(pitch("B3")));
        assert!(!c_major().contains(pitch("Bb3")));
    }

    #[test]
    fn degrees_wrap_into_neighbouring_octaves() {
        assert_eq!(c_major().pitch_at(0, 4), Ok(pitch("C4")));
        assert_eq!(c_major().pitch_at(4, 4), Ok(pitch("G4")));
        assert_eq!(c_major().pitch_at(7, 4), Ok(pitch("C5")));
        assert_eq!(c_major().pitch_at(-1, 4), Ok(pitch("B3")));
    }

    #[test]
    fn snap_moves_to_nearest_scale_pitch() {
        assert_eq!(c_major().snap(pitch("C#4")), pitch("C4"));
        assert_eq!(c_major().snap(pitch("E4")), pitch("E4"));
        let pentatonic = Scale::new(PitchClass::default(), ScaleKind::MinorPentatonic);
        assert_eq!(pentatonic.snap(pitch("A4")), pitch("A#4"));
    }
}
