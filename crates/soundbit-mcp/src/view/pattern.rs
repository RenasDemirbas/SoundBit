//! Pattern views, including a tracker-style grid.

use std::fmt::Write;

use serde_json::{Value, json};
use soundbit_app::model::{Pattern, PatternId, Song};

/// Width of one grid cell: the longest note name ("C#-1") plus a space.
const CELL: usize = 5;

/// Full details of a pattern, or `null` if it does not exist.
pub(crate) fn pattern_view(song: &Song, id: PatternId) -> Value {
    let Some(pattern) = song.pattern(id) else {
        return Value::Null;
    };
    let steps_per_bar = song.timing().steps_per_bar();
    let instrument = song.instrument(pattern.instrument).map(|i| i.name.as_str());
    let notes: Vec<Value> = pattern
        .notes()
        .iter()
        .map(|note| {
            json!({
                "note": note.pitch.to_string(),
                "start": note.start,
                "length": note.length.get(),
                "velocity": note.velocity.get(),
            })
        })
        .collect();
    let placed_at: Vec<Value> = song
        .tracks()
        .flat_map(|(track_id, track)| {
            track
                .clips()
                .filter(move |&(_, placed)| placed == id)
                .map(move |(bar, _)| json!({"track_id": track_id.get(), "bar": bar}))
        })
        .collect();
    json!({
        "id": id.get(),
        "name": pattern.name,
        "instrument": {"id": pattern.instrument.get(), "name": instrument},
        "steps_per_bar": steps_per_bar,
        "notes": notes,
        "grid": tracker_grid(pattern, steps_per_bar),
        "placed_at": placed_at,
    })
}

/// One row per step. Each column is a voice lane: a note name where a note starts, `|` while
/// it is held, `.` when silent. Chords use several lanes.
pub(crate) fn tracker_grid(pattern: &Pattern, steps_per_bar: u16) -> String {
    // Greedily assign each note to the first lane that is free at its start.
    let mut lane_ends: Vec<u32> = Vec::new();
    let mut placed = Vec::with_capacity(pattern.notes().len());
    for note in pattern.notes() {
        let start = u32::from(note.start);
        let lane = lane_ends
            .iter()
            .position(|&end| end <= start)
            .unwrap_or(lane_ends.len());
        if lane == lane_ends.len() {
            lane_ends.push(0);
        }
        lane_ends[lane] = note.end();
        placed.push((lane, note));
    }

    let lanes = lane_ends.len().max(1);
    let mut grid = String::new();
    for step in 0..steps_per_bar {
        let mut cells = vec![".".to_owned(); lanes];
        for (lane, note) in &placed {
            if note.start == step {
                cells[*lane] = note.pitch.to_string();
            } else if u32::from(note.start) < u32::from(step) && u32::from(step) < note.end() {
                "|".clone_into(&mut cells[*lane]);
            }
        }
        let _ = write!(grid, "{step:02} ");
        for cell in cells {
            let _ = write!(grid, "{cell:<CELL$}");
        }
        grid.truncate(grid.trim_end().len());
        grid.push('\n');
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use soundbit_app::model::{InstrumentId, Note};

    fn note(name: &str, start: u16, length: u16) -> Note {
        Note::new(name.parse().expect("valid"), start, length).expect("valid")
    }

    #[test]
    fn grid_shows_starts_holds_and_chords() {
        let mut pattern = Pattern::new("p", InstrumentId::new(1));
        pattern
            .set_notes(vec![note("C4", 0, 2), note("E4", 0, 1), note("G4", 2, 1)])
            .expect("valid");
        let grid = tracker_grid(&pattern, 4);
        assert_eq!(grid, "00 C4   E4\n01 |    .\n02 G4   .\n03 .    .\n");
    }

    #[test]
    fn empty_pattern_has_one_silent_lane() {
        let pattern = Pattern::new("p", InstrumentId::new(1));
        assert_eq!(tracker_grid(&pattern, 2), "00 .\n01 .\n");
    }
}
