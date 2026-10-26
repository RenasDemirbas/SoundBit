//! `soundbit mcp`: serve the Model Context Protocol over stdio or localhost HTTP.
//!
//! With stdio, standard output carries the protocol, so this command must never print to it.

use std::path::PathBuf;
use std::thread;

use soundbit_mcp::{HttpEndpoint, ServerOptions, SoundBitServer, run_stdio};

pub(crate) fn run(root: PathBuf, open: Option<PathBuf>, http: Option<u16>) -> anyhow::Result<()> {
    let options = ServerOptions { root, open };
    match http {
        None => run_stdio(options)?,
        Some(port) => {
            let endpoint = HttpEndpoint::start(SoundBitServer::new(options)?, port)?;
            eprintln!("SoundBit MCP endpoint: {} (Ctrl+C to stop)", endpoint.url());
            loop {
                thread::park();
            }
        }
    }
    Ok(())
}
