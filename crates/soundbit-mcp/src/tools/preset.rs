//! The preset library.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use soundbit_app::PresetLibrary;

use super::{ToolResult, attempt, reply};
use crate::SoundBitServer;

/// Which presets to list.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub(crate) struct PresetListInput {
    /// Only list this category, e.g. "strings", "brass", "drums", "lead", "bass". Lists
    /// everything when unset.
    category: Option<String>,
}

#[tool_router(router = preset_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Lists the instrument presets: id, name, category, voice type and what \
                       each sounds like and suits (including which notes to play drums on). Add \
                       one with instrument_add_preset.",
        annotations(read_only_hint = true)
    )]
    #[allow(clippy::unused_self, reason = "rmcp tools are methods")]
    fn preset_list(&self, Parameters(input): Parameters<PresetListInput>) -> ToolResult {
        let library = attempt!(PresetLibrary::builtin());
        let mut categories: Vec<&str> = library
            .all()
            .iter()
            .map(|preset| preset.category.as_str())
            .collect();
        categories.sort_unstable();
        categories.dedup();
        let presets: Vec<_> = library
            .all()
            .iter()
            .filter(|preset| {
                input
                    .category
                    .as_deref()
                    .is_none_or(|category| preset.category == category)
            })
            .map(|preset| {
                json!({
                    "id": preset.id,
                    "name": preset.instrument.name,
                    "category": preset.category,
                    "voice": preset.instrument.voice.kind(),
                    "description": preset.description,
                })
            })
            .collect();
        reply(&json!({"categories": categories, "presets": presets}))
    }
}
