//! Server start-up errors.

use std::io;
use std::path::PathBuf;

use soundbit_app::AppError;
use thiserror::Error;

/// The MCP server could not start or stopped with an error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ServeError {
    /// The workspace directory is missing or unreadable.
    #[error("workspace {} is not an accessible directory", path.display())]
    Workspace {
        /// The configured workspace.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },

    /// The project given at start-up could not be opened.
    #[error("cannot open the initial project")]
    Open(#[from] AppError),

    /// The HTTP endpoint could not listen on its port.
    #[error("cannot listen on 127.0.0.1:{port}; is another SoundBit already running?")]
    Bind {
        /// The requested port.
        port: u16,
        /// The underlying error.
        source: io::Error,
    },

    /// The async runtime could not start.
    #[error("cannot start the async runtime")]
    Runtime(#[from] io::Error),

    /// The MCP connection failed.
    #[error("MCP connection failed: {0}")]
    Protocol(String),
}
