//! Process-wide state shared by commands and the sync thread.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use soundbit_app::model::Song;
use soundbit_app::playback::player;
use soundbit_app::{Playback, Session, SharedSession};
use soundbit_audio_io::AudioOutput;
use soundbit_mcp::{HttpEndpoint, SoundBitServer};
use tauri::{AppHandle, Manager};

/// Port of the embedded MCP endpoint unless `SOUNDBIT_MCP_PORT` overrides it. Fixed, so MCP
/// client configuration keeps working across restarts.
const DEFAULT_MCP_PORT: u16 = 7413;

/// Audio output status. The output must stay alive for playback to work.
#[derive(Debug)]
pub(crate) struct Audio {
    _output: Option<AudioOutput>,
    pub(crate) device: Option<String>,
    pub(crate) error: Option<String>,
}

/// Embedded MCP endpoint status.
#[derive(Debug)]
pub(crate) struct Mcp {
    _endpoint: Option<HttpEndpoint>,
    pub(crate) url: Option<String>,
    pub(crate) error: Option<String>,
}

/// Everything the commands need. Lock order: session before playback.
pub(crate) struct AppState {
    session: SharedSession,
    playback: Mutex<Option<Playback>>,
    pub(crate) audio: Audio,
    pub(crate) mcp: Mcp,
    pub(crate) workspace: PathBuf,
}

impl AppState {
    /// Opens the audio device and the MCP endpoint. Failures are reported in the UI rather
    /// than preventing the app from starting.
    pub(crate) fn start(workspace: PathBuf) -> Self {
        let session: SharedSession = Arc::new(Mutex::new(Session::new(Song::new("Untitled"))));
        let (playback, audio) = open_audio();
        let mcp = open_mcp(&session, &workspace);
        Self {
            session,
            playback: Mutex::new(playback),
            audio,
            mcp,
            workspace,
        }
    }

    /// Locks the session. Commands apply atomically, so a poisoned lock is safe to recover.
    pub(crate) fn session(&self) -> MutexGuard<'_, Session> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Locks playback; `None` when no audio device is available.
    pub(crate) fn playback(&self) -> MutexGuard<'_, Option<Playback>> {
        self.playback.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn open_audio() -> (Option<Playback>, Audio) {
    let (control, mut rt) = player();
    match AudioOutput::open_default(move |buffer| rt.process(buffer)) {
        Ok(output) => {
            let info = output.info().clone();
            let playback = Playback::new(control, info.sample_rate);
            (
                Some(playback),
                Audio {
                    _output: Some(output),
                    device: Some(info.device),
                    error: None,
                },
            )
        }
        Err(error) => (
            None,
            Audio {
                _output: None,
                device: None,
                error: Some(error.to_string()),
            },
        ),
    }
}

fn open_mcp(session: &SharedSession, workspace: &std::path::Path) -> Mcp {
    let port = std::env::var("SOUNDBIT_MCP_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_MCP_PORT);
    let endpoint = SoundBitServer::shared(Arc::clone(session), workspace)
        .and_then(|server| HttpEndpoint::start(server, port));
    match endpoint {
        Ok(endpoint) => Mcp {
            url: Some(endpoint.url()),
            _endpoint: Some(endpoint),
            error: None,
        },
        Err(error) => Mcp {
            _endpoint: None,
            url: None,
            error: Some(error.to_string()),
        },
    }
}

/// The directory agents may read and write: `<Music>/SoundBit`, created if needed.
pub(crate) fn workspace_dir(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let paths = app.path();
    let base = paths.audio_dir().or_else(|_| paths.home_dir())?;
    let dir = base.join("SoundBit");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
