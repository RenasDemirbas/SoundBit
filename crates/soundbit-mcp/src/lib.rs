//! Model Context Protocol adapter.
//!
//! Exposes a SoundBit session to AI agents as MCP tools. Tool handlers only translate: they
//! parse input, build an app [`Command`](soundbit_app::Command) or read the song, and format
//! the result. No music logic lives here; it belongs in `soundbit-app` or `soundbit-core`.
//!
//! Every change goes through the session's [`Editor`](soundbit_app::Editor) with
//! [`Origin::Agent`](soundbit_app::Origin::Agent), so it is undoable and labelled `AI:` in the
//! user's history. File access is confined to a workspace directory.
//!
//! Two transports are provided: [`run_stdio`] for clients that launch `soundbit mcp`, and
//! [`HttpEndpoint`] for clients that connect to a running app over localhost HTTP.

#![forbid(unsafe_code)]

mod error;
mod guide;
mod http;
mod input;
mod server;
mod tools;
mod view;
mod workspace;

pub use error::ServeError;
pub use http::HttpEndpoint;
pub use server::{ServerOptions, SoundBitServer};

use rmcp::ServiceExt;
use rmcp::transport::stdio;

/// Serves MCP over standard input and output until the client disconnects.
///
/// Blocks the calling thread; it runs its own single-threaded async runtime.
///
/// # Errors
///
/// Returns [`ServeError`] if the workspace or initial project cannot be opened, or the
/// connection fails.
pub fn run_stdio(options: ServerOptions) -> Result<(), ServeError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let server = SoundBitServer::new(options)?;
        let service = server
            .serve(stdio())
            .await
            .map_err(|error| ServeError::Protocol(error.to_string()))?;
        service
            .waiting()
            .await
            .map_err(|error| ServeError::Protocol(error.to_string()))?;
        Ok(())
    })
}
