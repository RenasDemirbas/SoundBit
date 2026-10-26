//! Song and track views.

use std::fmt::Write;

use serde_json::{Value, json};
use soundbit_app::Session;
use soundbit_app::model::{Song, TrackId};

use super::problems;
use crate::workspace::Workspace;

/// Width of the track label column in the arrangement grid.
const LABEL: usize = 16;

/// Overview of the whole song: settings, entities, arrangement and history.
pub(crate) fn song_view(session: &Session, workspace: &Workspace) -> Value {
    let song = session.song();
    let timing = song.timing();
    let editor = session.editor();
    json!({
        "title": song.meta().title,
        "author": song.meta().author,
        "description": song.meta().description,
        "file": session.path().map(|path| workspace.display(path)),
        "unsaved_changes": session.is_modified(),
        "timing": {
            "bpm": timing.bpm.get(),
            "steps_per_beat": timing.steps_per_beat.get(),
            "beats_per_bar": timing.beats_per_bar.get(),
            "steps_per_bar": timing.steps_per_bar(),
            "swing": timing.swing.get(),
        },
        "scale": {"root": song.scale().root.to_string(), "kind": song.scale().kind},
        "chip_profile": song.chip_profile(),
        "reverb": {"size_percent": song.reverb().size.get(), "damping_percent": song.reverb().damping.get()},
        "loop": song.loop_range().map(|range| json!({"start_bar": range.start(), "end_bar": range.end()})),
        "length": {"bars": song.length_bars(), "seconds": (song.length_seconds() * 10.0).round() / 10.0},
        "instruments": song.instruments().map(|(id, instrument)| json!({
            "id": id.get(),
            "name": instrument.name,
            "voice": instrument.voice.kind(),
            "volume": instrument.volume.get(),
            "reverb_send_percent": instrument.reverb_send.get(),
        })).collect::<Vec<_>>(),
        "patterns": song.patterns().map(|(id, pattern)| json!({
            "id": id.get(),
            "name": pattern.name,
            "instrument_id": pattern.instrument.get(),
            "notes": pattern.notes().len(),
        })).collect::<Vec<_>>(),
        "tracks": song.tracks().map(|(id, _)| track_view(song, id)).collect::<Vec<_>>(),
        "arrangement": arrangement_grid(song),
        "profile_problems": problems(&song.profile_violations()),
        "undo": editor.undo_label(),
        "redo": editor.redo_label(),
    })
}

/// A track with its chip channel and clips, or `null` if it does not exist.
pub(crate) fn track_view(song: &Song, id: TrackId) -> Value {
    let Some(track) = song.track(id) else {
        return Value::Null;
    };
    let position = song.tracks().position(|(track_id, _)| track_id == id);
    let channel = song
        .chip_profile()
        .channels()
        .and_then(|channels| position.and_then(|index| channels.get(index)))
        .map(|channel| channel.name);
    let settings = track.settings();
    json!({
        "id": id.get(),
        "name": settings.name,
        "channel": channel,
        "muted": settings.muted,
        "volume": settings.volume.get(),
        "clips": track.clips().map(|(bar, pattern)| json!({"bar": bar, "pattern_id": pattern.get()})).collect::<Vec<_>>(),
    })
}

/// Tracks as rows and bars as columns, showing the pattern id in each bar (`.` for empty).
pub(crate) fn arrangement_grid(song: &Song) -> String {
    let bars = song.length_bars();
    let mut grid = format!("{:<LABEL$}", "bar");
    for bar in 0..bars {
        let _ = write!(grid, "{bar:>4}");
    }
    grid.push('\n');
    for (id, track) in song.tracks() {
        let label: String = format!("T{} {}", id.get(), track.settings().name)
            .chars()
            .take(LABEL - 1)
            .collect();
        let _ = write!(grid, "{label:<LABEL$}");
        for bar in 0..bars {
            match track.clip_at(bar) {
                Some(pattern) => {
                    let _ = write!(grid, "{:>4}", pattern.get());
                }
                None => grid.push_str("   ."),
            }
        }
        grid.push('\n');
    }
    grid
}
