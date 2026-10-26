//! Instrument preset files: ready-made sounds stored as data.
//!
//! ```json
//! { "format": "soundbit-presets", "format_version": 1, "source": "...",
//!   "presets": [ { "id": "fm_brass", "category": "brass", "description": "...",
//!                  "instrument": { "name": "Brass", "voice": { ... } } } ] }
//! ```
//!
//! The `instrument` object has the same shape as an instrument in a project file, so a preset
//! is validated exactly like a song's instrument.

use serde::{Deserialize, Serialize};
use soundbit_core::Instrument;

use crate::FormatError;

/// Newest preset file version.
pub const PRESET_FORMAT_VERSION: u32 = 1;

const PRESET_TAG: &str = "soundbit-presets";

/// A named, ready-made instrument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preset {
    /// Stable identifier in `snake_case`, e.g. `"fm_brass"`.
    pub id: String,
    /// Group for browsing, e.g. `"brass"` or `"drums"`.
    pub category: String,
    /// What the preset sounds like and what it suits.
    pub description: String,
    /// Where the sound comes from (author or reference), copied from the file.
    #[serde(default)]
    pub source: String,
    /// The instrument itself.
    pub instrument: Instrument,
}

#[derive(Deserialize)]
struct PresetFile {
    format: Option<String>,
    format_version: Option<u32>,
    source: String,
    presets: Vec<Preset>,
}

/// Reads a preset file. Each preset's `source` is set to the file's `source`.
///
/// # Errors
///
/// Returns [`FormatError::NotAPresetFile`] if the format tag is missing,
/// [`FormatError::UnsupportedVersion`] for unknown versions, and [`FormatError::Json`] for
/// malformed JSON or invalid instruments.
pub fn parse_presets(text: &str) -> Result<Vec<Preset>, FormatError> {
    let file: PresetFile = serde_json::from_str(text)?;
    if file.format.as_deref() != Some(PRESET_TAG) {
        return Err(FormatError::NotAPresetFile);
    }
    match file.format_version {
        Some(PRESET_FORMAT_VERSION) => Ok(file
            .presets
            .into_iter()
            .map(|preset| Preset {
                source: file.source.clone(),
                ..preset
            })
            .collect()),
        found => Err(FormatError::UnsupportedVersion {
            found: found.unwrap_or(0),
            supported: PRESET_FORMAT_VERSION,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundbit_core::Voice;

    #[test]
    fn reads_presets_with_the_file_source() {
        let text = r#"{"format": "soundbit-presets", "format_version": 1, "source": "Tests",
            "presets": [{"id": "sub", "category": "bass", "description": "Sine sub",
                         "instrument": {"name": "Sub", "voice": {"type": "sine"}}}]}"#;
        let presets = parse_presets(text).expect("valid");
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].source, "Tests");
        assert_eq!(presets[0].instrument.voice, Voice::Sine);
    }

    #[test]
    fn rejects_other_files_and_invalid_instruments() {
        assert!(matches!(
            parse_presets(r#"{"source": "x", "presets": []}"#),
            Err(FormatError::NotAPresetFile)
        ));
        let text = r#"{"format": "soundbit-presets", "format_version": 1, "source": "x",
            "presets": [{"id": "a", "category": "b", "description": "c",
                         "instrument": {"name": "A", "voice": {"type": "sine"}, "volume": 400}}]}"#;
        assert!(matches!(parse_presets(text), Err(FormatError::Json(_))));
    }
}
