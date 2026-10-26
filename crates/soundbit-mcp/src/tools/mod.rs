//! MCP tools, one file per domain. Names follow `noun_verb`.
//!
//! Handlers never fail at the protocol level: problems are returned as tool errors
//! (`is_error: true`) with a message the agent can act on.

mod files;
mod history;
mod instrument;
mod pattern;
mod preset;
mod reference;
mod song;
mod track;

use std::fmt::Display;

use rmcp::ErrorData;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;
use soundbit_app::{Command, Created, Origin, Outcome, Session};

use crate::SoundBitServer;
use crate::view::problems;

/// What every tool returns.
pub(crate) type ToolResult = Result<CallToolResult, ErrorData>;

/// All tools.
pub(crate) fn router() -> ToolRouter<SoundBitServer> {
    SoundBitServer::reference_tools()
        + SoundBitServer::song_tools()
        + SoundBitServer::instrument_tools()
        + SoundBitServer::preset_tools()
        + SoundBitServer::pattern_tools()
        + SoundBitServer::track_tools()
        + SoundBitServer::history_tools()
        + SoundBitServer::file_tools()
}

/// A successful result carrying pretty-printed JSON.
#[allow(clippy::unnecessary_wraps, reason = "tools return `ToolResult`")]
pub(crate) fn reply(body: &Value) -> ToolResult {
    let text = serde_json::to_string_pretty(body).unwrap_or_else(|error| error.to_string());
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// A tool error the agent sees and can react to.
#[allow(clippy::unnecessary_wraps, reason = "tools return `ToolResult`")]
pub(crate) fn refuse(message: impl Display) -> ToolResult {
    Ok(CallToolResult::error(vec![ContentBlock::text(
        message.to_string(),
    )]))
}

/// Unwraps a result or returns it from the tool as a [`refuse`]d call.
macro_rules! attempt {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => return $crate::tools::refuse(error),
        }
    };
}
pub(crate) use attempt;

/// The first entity of a kind a command created.
pub(crate) fn created<T>(outcome: &Outcome, pick: impl Fn(&Created) -> Option<T>) -> Option<T> {
    outcome.created.iter().find_map(pick)
}

impl SoundBitServer {
    /// Runs a command on behalf of the agent and replies with a view of the result, the
    /// history label and any chip-profile problems.
    pub(crate) fn apply(
        &self,
        command: impl Into<Command>,
        view: impl FnOnce(&Session, &Outcome) -> Value,
    ) -> ToolResult {
        let mut session = self.session();
        let outcome = attempt!(session.editor_mut().execute(command, Origin::Agent));
        let mut body = view(&session, &outcome);
        if let Value::Object(map) = &mut body {
            map.insert("change".to_owned(), Value::String(outcome.label.clone()));
            if !outcome.violations.is_empty() {
                map.insert("profile_problems".to_owned(), problems(&outcome.violations));
            }
        }
        reply(&body)
    }
}
