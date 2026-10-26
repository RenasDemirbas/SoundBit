//! Tool outputs: JSON views of the song, shaped for agents to read.

mod instrument;
mod pattern;
mod song;

pub(crate) use instrument::instrument_view;
pub(crate) use pattern::pattern_view;
pub(crate) use song::{arrangement_grid, song_view, track_view};

use serde_json::{Value, json};
use soundbit_app::model::ProfileViolation;

/// Profile violations as plain sentences.
pub(crate) fn problems(violations: &[ProfileViolation]) -> Value {
    json!(
        violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    )
}
