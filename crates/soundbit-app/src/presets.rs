//! The built-in instrument presets.
//!
//! Presets are data, not code: they live as JSON files in `assets/presets/` and are embedded
//! into the binary at compile time, so every front end offers the same library without
//! having to find files at run time.

use soundbit_formats::{Preset, parse_presets};

use crate::AppError;

/// The preset files, by name.
const FILES: [(&str, &str); 4] = [
    (
        "orchestral.json",
        include_str!("../../../assets/presets/orchestral.json"),
    ),
    (
        "drums.json",
        include_str!("../../../assets/presets/drums.json"),
    ),
    (
        "chip.json",
        include_str!("../../../assets/presets/chip.json"),
    ),
    ("fm.json", include_str!("../../../assets/presets/fm.json")),
];

/// A searchable collection of presets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetLibrary {
    presets: Vec<Preset>,
}

impl PresetLibrary {
    /// The presets shipped with SoundBit.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Presets`] if a built-in file is invalid, which tests rule out.
    pub fn builtin() -> Result<Self, AppError> {
        let mut presets = Vec::new();
        for (file, text) in FILES {
            presets
                .extend(parse_presets(text).map_err(|source| AppError::Presets { file, source })?);
        }
        Ok(Self { presets })
    }

    /// Every preset, grouped by file and in file order.
    pub fn all(&self) -> &[Preset] {
        &self.presets
    }

    /// The preset with this id.
    pub fn find(&self, id: &str) -> Option<&Preset> {
        self.presets.iter().find(|preset| preset.id == id)
    }

    /// Ids of all presets, for error messages that list the choices.
    pub fn ids(&self) -> Vec<&str> {
        self.presets
            .iter()
            .map(|preset| preset.id.as_str())
            .collect()
    }
}
