//! Tool inputs: JSON shapes designed for agents, converted into domain values.
//!
//! These types are the public MCP schema. They are deliberately separate from the domain types,
//! so the model can evolve without breaking agents. Conversions validate and produce messages
//! that name the offending field.

mod choice;
mod note;
mod shaping;
mod voice;

pub(crate) use choice::Choice;
pub(crate) use note::{NoteInput, NoteRef, notes};
pub(crate) use shaping::{FilterInput, PitchEnvelopeInput, UnisonInput, VibratoInput};
pub(crate) use voice::{EnvelopeInput, VoiceInput};

use soundbit_app::model::ModelError;

/// Converts an optional raw number into a validated domain value.
pub(crate) fn optional<T, R>(value: Option<R>) -> Result<Option<T>, String>
where
    T: TryFrom<R, Error = ModelError>,
{
    value
        .map(T::try_from)
        .transpose()
        .map_err(|error| error.to_string())
}

/// Bars selected either explicitly or as a run of consecutive bars.
pub(crate) fn bar_selection(
    bars: Option<Vec<u16>>,
    start_bar: Option<u16>,
    count: Option<u16>,
) -> Result<Vec<u16>, String> {
    match (bars, start_bar) {
        (Some(bars), None) if !bars.is_empty() => Ok(bars),
        (None, Some(start)) => {
            let count = count.unwrap_or(1);
            Ok((0..count)
                .filter_map(|offset| start.checked_add(offset))
                .collect())
        }
        _ => Err(
            "give either `bars` (a non-empty list) or `start_bar` (with optional `count`)"
                .to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_bars_by_list_or_range() {
        assert_eq!(bar_selection(Some(vec![3, 1]), None, None), Ok(vec![3, 1]));
        assert_eq!(bar_selection(None, Some(4), Some(3)), Ok(vec![4, 5, 6]));
        assert_eq!(bar_selection(None, Some(2), None), Ok(vec![2]));
        assert!(bar_selection(None, None, None).is_err());
        assert!(bar_selection(Some(vec![1]), Some(1), None).is_err());
        assert!(bar_selection(Some(vec![]), None, None).is_err());
    }
}
