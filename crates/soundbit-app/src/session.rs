//! Editing sessions: one open song and where it is stored.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use soundbit_core::Song;
use soundbit_engine::{CHANNELS, RenderOptions, render};
use soundbit_formats::{FormatError, from_project_json, to_project_json, write_wav};

use crate::{AppError, Editor};

/// A session shared between adapters, e.g. the GUI and an MCP endpoint editing the same song.
pub type SharedSession = Arc<Mutex<Session>>;

/// Summary of a finished WAV export.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportReport {
    /// Length of the exported audio in seconds.
    pub duration_seconds: f64,
    /// Largest absolute sample value, in `[0, 1]`.
    pub peak: f32,
    /// Root-mean-square level of the whole mix, in `[0, 1]`.
    pub rms: f32,
    /// Level of every track's contribution, in track order.
    pub tracks: Vec<TrackLevel>,
}

/// How loud one track is in an export, measured before the master limiter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackLevel {
    /// Largest absolute sample value.
    pub peak: f32,
    /// Root-mean-square level over the whole export.
    pub rms: f32,
}

/// An open song being edited, and the project file it belongs to, if any.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    editor: Editor,
    path: Option<PathBuf>,
    saved_revision: Option<u64>,
}

impl Session {
    /// Starts a session for an unsaved song.
    pub fn new(song: Song) -> Self {
        Self {
            editor: Editor::new(song),
            path: None,
            saved_revision: None,
        }
    }

    /// Opens a project file.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Read`] if the file cannot be read and [`AppError::Format`] if it is
    /// not a valid project.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| AppError::Read {
            path: path.to_owned(),
            source,
        })?;
        let song = from_project_json(&text).map_err(|source| format_error(path, source))?;
        Ok(Self {
            editor: Editor::new(song),
            path: Some(path.to_owned()),
            saved_revision: Some(0),
        })
    }

    /// The open song.
    pub fn song(&self) -> &Song {
        self.editor.song()
    }

    /// The editor, for reading history and revision.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// The editor, for executing commands and undo/redo.
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    /// Whether the song changed since it was opened or last saved. A new, never-saved song
    /// counts as modified once it has been edited.
    pub fn is_modified(&self) -> bool {
        match self.saved_revision {
            Some(saved) => saved != self.editor.revision(),
            None => self.editor.revision() > 0,
        }
    }

    /// The project file, if the song has been opened or saved.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Saves the song to `path` and remembers it as the project file.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Write`] if the file cannot be written.
    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let path = path.as_ref();
        let text = to_project_json(self.song()).map_err(|source| format_error(path, source))?;
        fs::write(path, text).map_err(|source| AppError::Write {
            path: path.to_owned(),
            source,
        })?;
        self.path = Some(path.to_owned());
        self.saved_revision = Some(self.editor.revision());
        Ok(())
    }

    /// Renders the song and writes it as a 16-bit WAV file.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Engine`] for invalid render options and [`AppError::Write`] or
    /// [`AppError::Format`] if the file cannot be written.
    pub fn export_wav(
        &self,
        path: impl AsRef<Path>,
        options: &RenderOptions,
    ) -> Result<ExportReport, AppError> {
        let path = path.as_ref();
        let rendered = render(self.song(), options)?;
        let file = File::create(path).map_err(|source| AppError::Write {
            path: path.to_owned(),
            source,
        })?;
        write_wav(
            BufWriter::new(file),
            &rendered.samples,
            CHANNELS,
            rendered.sample_rate,
        )
        .map_err(|source| format_error(path, source))?;
        Ok(ExportReport {
            duration_seconds: rendered.duration_seconds(),
            peak: rendered.peak(),
            rms: root(rendered.mean_square()),
            tracks: rendered
                .tracks
                .iter()
                .map(|track| TrackLevel {
                    peak: track.peak,
                    rms: root(track.mean_square),
                })
                .collect(),
        })
    }
}

/// Square root of a mean square. Reports are not audio, so they may use `sqrt`.
#[allow(
    clippy::cast_possible_truncation,
    reason = "levels are in [0, 1], far inside f32 range"
)]
fn root(mean_square: f64) -> f32 {
    mean_square.sqrt() as f32
}

fn format_error(path: &Path, source: FormatError) -> AppError {
    match source {
        FormatError::Io(source) => AppError::Write {
            path: path.to_owned(),
            source,
        },
        source => AppError::Format {
            path: path.to_owned(),
            source,
        },
    }
}
