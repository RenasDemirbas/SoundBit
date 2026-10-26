//! The SoundBit project file: a versioned JSON envelope around a song.
//!
//! ```json
//! { "format": "soundbit-project", "format_version": 2, "song": { ... } }
//! ```
//!
//! Every change to the song's JSON layout bumps [`FORMAT_VERSION`] and ships a migration from
//! the previous version (see `migrations`), so old projects keep loading.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use soundbit_core::Song;

use crate::FormatError;
use crate::migrations::{OLDEST_VERSION, upgrade};

/// File extension of project files, without the dot.
pub const PROJECT_EXTENSION: &str = "sbproj";

/// Newest project format version, written by [`to_project_json`].
pub const FORMAT_VERSION: u32 = 2;

const FORMAT_TAG: &str = "soundbit-project";

#[derive(Serialize)]
struct ProjectRef<'a> {
    format: &'static str,
    format_version: u32,
    song: &'a Song,
}

#[derive(Deserialize)]
struct Header {
    format: Option<String>,
    format_version: Option<u32>,
}

#[derive(Deserialize)]
struct Project {
    song: Song,
}

/// Serializes a song as a pretty-printed project file, ending with a newline.
///
/// # Errors
///
/// Returns [`FormatError::Json`] if serialization fails, which does not happen for valid songs.
pub fn to_project_json(song: &Song) -> Result<String, FormatError> {
    let project = ProjectRef {
        format: FORMAT_TAG,
        format_version: FORMAT_VERSION,
        song,
    };
    let mut text = serde_json::to_string_pretty(&project)?;
    text.push('\n');
    Ok(text)
}

/// Reads a song from a project file, upgrading older format versions.
///
/// # Errors
///
/// Returns [`FormatError::NotAProject`] if the format tag is missing,
/// [`FormatError::UnsupportedVersion`] for unknown versions, and [`FormatError::Json`] for
/// malformed JSON or an invalid song (the message includes the line and the model's reason).
pub fn from_project_json(text: &str) -> Result<Song, FormatError> {
    let header: Header = serde_json::from_str(text)?;
    if header.format.as_deref() != Some(FORMAT_TAG) {
        return Err(FormatError::NotAProject);
    }
    match header.format_version {
        // Parsing the text directly keeps line numbers in error messages.
        Some(FORMAT_VERSION) => Ok(serde_json::from_str::<Project>(text)?.song),
        Some(version) if (OLDEST_VERSION..FORMAT_VERSION).contains(&version) => {
            let mut project: Value = serde_json::from_str(text)?;
            if let Some(song) = project.get_mut("song") {
                upgrade(song, version);
            }
            Ok(serde_json::from_value::<Project>(project)?.song)
        }
        found => Err(FormatError::UnsupportedVersion {
            found: found.unwrap_or(0),
            supported: FORMAT_VERSION,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_song() {
        let song = Song::new("Round trip");
        let text = to_project_json(&song).expect("serializes");
        assert!(
            text.starts_with("{\n  \"format\": \"soundbit-project\",\n  \"format_version\": 2,")
        );
        assert_eq!(from_project_json(&text).expect("loads"), song);
    }

    #[test]
    fn rejects_other_json() {
        assert!(matches!(
            from_project_json("{\"song\": {}}"),
            Err(FormatError::NotAProject)
        ));
        assert!(matches!(
            from_project_json("[1, 2]"),
            Err(FormatError::Json(_))
        ));
    }

    #[test]
    fn rejects_unknown_versions() {
        let text = r#"{"format": "soundbit-project", "format_version": 99, "song": {}}"#;
        assert!(matches!(
            from_project_json(text),
            Err(FormatError::UnsupportedVersion {
                found: 99,
                supported: FORMAT_VERSION
            })
        ));
    }

    #[test]
    fn invalid_song_errors_point_at_the_problem() {
        let text = r#"{"format": "soundbit-project", "format_version": 2, "song": {
            "instruments": [{"id": 1, "name": "x", "voice": {"type": "sine"}, "volume": 300}]
        }}"#;
        let error = from_project_json(text).expect_err("invalid volume");
        let cause = std::error::Error::source(&error)
            .expect("has a cause")
            .to_string();
        assert!(cause.contains("line 2"), "{cause}");
    }
}
