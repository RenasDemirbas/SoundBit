//! Primitive, reversible changes to a song.
//!
//! Every [`Edit`] maps onto one `Song` method and, when applied, returns its exact inverse.
//! Undo applies the inverse; applying the inverse of the inverse is redo.

use soundbit_core::{
    ChipProfile, Instrument, InstrumentId, LoopRange, ModelError, Pattern, PatternId, Removed,
    Reverb, Scale, Song, SongMeta, TimeSettings, Track, TrackId, TrackSettings,
};

/// An entity created by a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Created {
    /// A new instrument.
    Instrument(InstrumentId),
    /// A new pattern.
    Pattern(PatternId),
    /// A new track.
    Track(TrackId),
}

/// A primitive change to a song.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Edit {
    Meta(SongMeta),
    Timing(TimeSettings),
    Scale(Scale),
    ChipProfile(ChipProfile),
    LoopRange(Option<LoopRange>),
    Reverb(Reverb),
    AddInstrument(Instrument),
    ReplaceInstrument(InstrumentId, Instrument),
    RemoveInstrument(InstrumentId),
    RestoreInstrument(Removed<InstrumentId, Instrument>),
    AddPattern(Pattern),
    ReplacePattern(PatternId, Pattern),
    RemovePattern(PatternId),
    RestorePattern(Removed<PatternId, Pattern>),
    AddTrack(TrackSettings),
    TrackSettings(TrackId, TrackSettings),
    RemoveTrack(TrackId),
    RestoreTrack(Removed<TrackId, Track>),
    /// Places a pattern at a bar, or clears the bar when `pattern` is `None`.
    Clip {
        track: TrackId,
        bar: u16,
        pattern: Option<PatternId>,
    },
    /// Several edits applied in order.
    Batch(Vec<Edit>),
}

/// The result of applying an edit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Applied {
    /// Applying this undoes the edit.
    pub(crate) inverse: Edit,
    /// Entities the edit created.
    pub(crate) created: Vec<Created>,
}

impl Applied {
    fn undo_with(inverse: Edit) -> Self {
        Self {
            inverse,
            created: Vec::new(),
        }
    }

    /// An entity was added; undo removes it again.
    fn added<I: Copy>(id: I, remove: fn(I) -> Edit, created: fn(I) -> Created) -> Self {
        Self {
            inverse: remove(id),
            created: vec![created(id)],
        }
    }

    /// A removed entity was put back; undo removes it again.
    fn restored<I: Copy, T>(
        removed: Removed<I, T>,
        restore: impl FnOnce(Removed<I, T>) -> Result<(), ModelError>,
        remove: fn(I) -> Edit,
    ) -> Result<Self, ModelError> {
        let id = removed.id;
        restore(removed)?;
        Ok(Self::undo_with(remove(id)))
    }
}

impl Edit {
    /// Applies the edit. On error the song may be partly changed, so callers apply edits to a
    /// draft copy and discard it on failure.
    pub(crate) fn apply(self, song: &mut Song) -> Result<Applied, ModelError> {
        let undo = Applied::undo_with;
        Ok(match self {
            Self::Meta(meta) => undo(Self::Meta(song.set_meta(meta))),
            Self::Timing(timing) => undo(Self::Timing(song.set_timing(timing)?)),
            Self::Scale(scale) => undo(Self::Scale(song.set_scale(scale))),
            Self::ChipProfile(profile) => undo(Self::ChipProfile(song.set_chip_profile(profile))),
            Self::LoopRange(range) => undo(Self::LoopRange(song.set_loop_range(range))),
            Self::Reverb(reverb) => undo(Self::Reverb(song.set_reverb(reverb))),

            Self::AddInstrument(instrument) => Applied::added(
                song.add_instrument(instrument)?,
                Self::RemoveInstrument,
                Created::Instrument,
            ),
            Self::ReplaceInstrument(id, instrument) => undo(Self::ReplaceInstrument(
                id,
                song.replace_instrument(id, instrument)?,
            )),
            Self::RemoveInstrument(id) => {
                undo(Self::RestoreInstrument(song.remove_instrument(id)?))
            }
            Self::RestoreInstrument(removed) => Applied::restored(
                removed,
                |removed| song.restore_instrument(removed),
                Self::RemoveInstrument,
            )?,

            Self::AddPattern(pattern) => Applied::added(
                song.add_pattern(pattern)?,
                Self::RemovePattern,
                Created::Pattern,
            ),
            Self::ReplacePattern(id, pattern) => {
                undo(Self::ReplacePattern(id, song.replace_pattern(id, pattern)?))
            }
            Self::RemovePattern(id) => undo(Self::RestorePattern(song.remove_pattern(id)?)),
            Self::RestorePattern(removed) => Applied::restored(
                removed,
                |removed| song.restore_pattern(removed),
                Self::RemovePattern,
            )?,

            Self::AddTrack(settings) => {
                Applied::added(song.add_track(settings)?, Self::RemoveTrack, Created::Track)
            }
            Self::TrackSettings(id, settings) => undo(Self::TrackSettings(
                id,
                song.set_track_settings(id, settings)?,
            )),
            Self::RemoveTrack(id) => undo(Self::RestoreTrack(song.remove_track(id)?)),
            Self::RestoreTrack(removed) => Applied::restored(
                removed,
                |removed| song.restore_track(removed),
                Self::RemoveTrack,
            )?,

            Self::Clip {
                track,
                bar,
                pattern,
            } => {
                let previous = match pattern {
                    Some(pattern) => song.place_clip(track, bar, pattern)?,
                    None => song.clear_clip(track, bar)?,
                };
                undo(Self::Clip {
                    track,
                    bar,
                    pattern: previous,
                })
            }
            Self::Batch(edits) => apply_batch(edits, song)?,
        })
    }
}

/// Applies edits in order; the inverse undoes them in reverse order.
pub(crate) fn apply_batch(edits: Vec<Edit>, song: &mut Song) -> Result<Applied, ModelError> {
    let mut inverses = Vec::with_capacity(edits.len());
    let mut created = Vec::new();
    for edit in edits {
        let applied = edit.apply(song)?;
        inverses.push(applied.inverse);
        created.extend(applied.created);
    }
    inverses.reverse();
    Ok(Applied {
        inverse: Edit::Batch(inverses),
        created,
    })
}
