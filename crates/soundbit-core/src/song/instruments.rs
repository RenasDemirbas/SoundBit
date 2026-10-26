//! Instrument operations on [`Song`].

use super::{Song, check_capacity};
use crate::limits::MAX_INSTRUMENTS;
use crate::{Instrument, InstrumentId, ModelError, Removed};

impl Song {
    /// Instruments in display order.
    pub fn instruments(&self) -> impl Iterator<Item = (InstrumentId, &Instrument)> {
        self.instruments.iter()
    }

    /// The instrument with this id.
    pub fn instrument(&self, id: InstrumentId) -> Option<&Instrument> {
        self.instruments.get(id)
    }

    /// Adds an instrument and returns its new id.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::LimitReached`] if the song already has the maximum number of
    /// instruments.
    pub fn add_instrument(&mut self, instrument: Instrument) -> Result<InstrumentId, ModelError> {
        check_capacity(self.instruments.len(), MAX_INSTRUMENTS, "instruments")?;
        Ok(self.instruments.push(instrument))
    }

    /// Replaces an instrument and returns the old one.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownInstrument`] if there is no such instrument.
    pub fn replace_instrument(
        &mut self,
        id: InstrumentId,
        instrument: Instrument,
    ) -> Result<Instrument, ModelError> {
        self.instruments
            .replace(id, instrument)
            .ok_or(ModelError::UnknownInstrument(id))
    }

    /// Removes an instrument that no pattern uses.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::UnknownInstrument`] if there is no such instrument and
    /// [`ModelError::InstrumentInUse`] if a pattern still uses it.
    pub fn remove_instrument(
        &mut self,
        id: InstrumentId,
    ) -> Result<Removed<InstrumentId, Instrument>, ModelError> {
        if let Some((pattern, _)) = self.patterns.iter().find(|(_, p)| p.instrument == id) {
            return Err(ModelError::InstrumentInUse {
                instrument: id,
                pattern,
            });
        }
        self.instruments
            .remove(id)
            .ok_or(ModelError::UnknownInstrument(id))
    }

    /// Puts a removed instrument back under its old id and position.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::DuplicateId`] if the id is taken again and
    /// [`ModelError::LimitReached`] if the song is full.
    pub fn restore_instrument(
        &mut self,
        removed: Removed<InstrumentId, Instrument>,
    ) -> Result<(), ModelError> {
        check_capacity(self.instruments.len(), MAX_INSTRUMENTS, "instruments")?;
        self.instruments
            .restore(removed)
            .map_err(|id| ModelError::DuplicateId(id.to_string()))
    }
}
