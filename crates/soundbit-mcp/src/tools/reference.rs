//! Reference information for agents.

use rmcp::{tool, tool_router};

use super::{ToolResult, reply};
use crate::{SoundBitServer, guide};

#[tool_router(router = reference_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Explains the song model: note format, every voice type and its \
                       parameters, envelope ranges, chip profiles with their channels, scales and \
                       size limits. Call it once before composing.",
        annotations(read_only_hint = true)
    )]
    #[allow(clippy::unused_self, reason = "rmcp tools are methods")]
    fn reference_get(&self) -> ToolResult {
        reply(&guide::reference())
    }
}
