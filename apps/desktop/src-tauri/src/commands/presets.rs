//! The preset library for the instrument panel.

use serde::Serialize;
use soundbit_app::PresetLibrary;
use soundbit_app::model::VoiceKind;

use super::CommandResult;

/// A preset as the front end lists it. Mirrored by `PresetInfo` in `src/lib/ipc/types.ts`.
#[derive(Debug, Serialize)]
pub(crate) struct PresetInfo {
    id: String,
    name: String,
    category: String,
    description: String,
    voice: VoiceKind,
}

#[tauri::command]
pub(crate) fn list_presets() -> CommandResult<Vec<PresetInfo>> {
    let library = PresetLibrary::builtin().map_err(|error| error.to_string())?;
    Ok(library
        .all()
        .iter()
        .map(|preset| PresetInfo {
            id: preset.id.clone(),
            name: preset.instrument.name.clone(),
            category: preset.category.clone(),
            description: preset.description.clone(),
            voice: preset.instrument.voice.kind(),
        })
        .collect())
}
