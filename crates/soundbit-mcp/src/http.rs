//! MCP over streamable HTTP on localhost, for clients that connect to a running app.
//!
//! The endpoint listens on `127.0.0.1` only. `rmcp` rejects requests whose `Host` is not a
//! loopback name (DNS rebinding), and Origin validation is enforced with an empty allow-list,
//! so any request from a web page (which always carries `Origin`) is refused. Native MCP
//! clients send no `Origin` and are accepted.

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio_util::sync::CancellationToken;

use crate::{ServeError, SoundBitServer};

/// URL path of the MCP endpoint.
const PATH: &str = "/mcp";

/// A running HTTP endpoint. Dropping it shuts the server down.
#[derive(Debug)]
pub struct HttpEndpoint {
    address: SocketAddr,
    cancel: CancellationToken,
    thread: Option<JoinHandle<()>>,
}

impl HttpEndpoint {
    /// Serves `server` on `127.0.0.1:port` (port 0 picks a free port) from a background
    /// thread.
    ///
    /// # Errors
    ///
    /// Returns [`ServeError::Bind`] if the port is taken and [`ServeError::Runtime`] if the
    /// server thread cannot start.
    pub fn start(server: SoundBitServer, port: u16) -> Result<Self, ServeError> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .map_err(|source| ServeError::Bind { port, source })?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let thread = thread::Builder::new()
            .name("soundbit-mcp-http".to_owned())
            .spawn(move || {
                if let Err(error) = serve(listener, server, &token) {
                    eprintln!("soundbit: MCP endpoint stopped: {error}");
                }
            })?;
        Ok(Self {
            address,
            cancel,
            thread: Some(thread),
        })
    }

    /// The address the endpoint listens on.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// The URL to configure in MCP clients.
    pub fn url(&self) -> String {
        format!("http://{}{PATH}", self.address)
    }
}

impl Drop for HttpEndpoint {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve(
    listener: TcpListener,
    server: SoundBitServer,
    cancel: &CancellationToken,
) -> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        let config = StreamableHttpServerConfig::default()
            .enforce_origin_validation()
            .with_cancellation_token(cancel.child_token());
        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );
        let router = axum::Router::new().nest_service(PATH, service);
        axum::serve(listener, router)
            .with_graceful_shutdown(cancel.clone().cancelled_owned())
            .await
    })
}
