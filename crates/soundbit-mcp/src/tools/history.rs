//! Undo and redo.

use rmcp::{tool, tool_router};
use serde_json::json;

use super::{ToolResult, attempt, refuse, reply};
use crate::SoundBitServer;

#[tool_router(router = history_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Undoes the most recent change (yours or the user's) and returns what was \
                          undone."
    )]
    fn history_undo(&self) -> ToolResult {
        let mut session = self.session();
        match attempt!(session.editor_mut().undo()) {
            Some(label) => {
                reply(&json!({"undone": label, "next_undo": session.editor().undo_label()}))
            }
            None => refuse("nothing to undo"),
        }
    }

    #[tool(description = "Redoes the most recently undone change and returns what was redone.")]
    fn history_redo(&self) -> ToolResult {
        let mut session = self.session();
        match attempt!(session.editor_mut().redo()) {
            Some(label) => {
                reply(&json!({"redone": label, "next_redo": session.editor().redo_label()}))
            }
            None => refuse("nothing to redo"),
        }
    }
}
