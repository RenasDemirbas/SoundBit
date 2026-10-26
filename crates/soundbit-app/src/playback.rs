//! Live playback of the editor's song.
//!
//! The composition root creates a [`player`] pair, hands the [`RtPlayer`] to an audio output
//! and wraps the [`PlayerControl`] in a [`Playback`]. Calling [`Playback::sync`] regularly
//! sends edits to the audio thread while the song plays.

use soundbit_engine::{PositionMap, Schedule};

pub use soundbit_engine::realtime::{PlayerControl, PlayerError, RtPlayer, player};

use crate::{Editor, PlaybackError};

/// Plays the editor's song through a real-time player.
pub struct Playback {
    control: PlayerControl,
    sample_rate: u32,
    /// `(editor id, revision)` of the song the audio thread is playing, while playing.
    version: Option<(u64, u64)>,
    positions: Option<PositionMap>,
}

impl Playback {
    /// Wraps a player whose audio output runs at `sample_rate`.
    pub fn new(control: PlayerControl, sample_rate: u32) -> Self {
        Self {
            control,
            sample_rate,
            version: None,
            positions: None,
        }
    }

    /// Output sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Starts playing the song from `from_bar`. With `looping`, the song's loop region (or the
    /// whole song) repeats until stopped.
    ///
    /// # Errors
    ///
    /// Returns [`PlaybackError`] if the song cannot be compiled or the audio thread does not
    /// respond.
    pub fn play(
        &mut self,
        editor: &Editor,
        from_bar: u16,
        looping: bool,
    ) -> Result<(), PlaybackError> {
        let schedule = Schedule::compile(editor.song(), self.sample_rate)?;
        let positions = schedule.position_map();
        self.control
            .start(schedule, positions.bar_start(from_bar), looping)?;
        self.version = Some(version(editor));
        self.positions = Some(positions);
        Ok(())
    }

    /// Stops playback.
    ///
    /// # Errors
    ///
    /// Returns [`PlaybackError::Player`] if the audio thread does not respond.
    pub fn stop(&mut self) -> Result<(), PlaybackError> {
        self.version = None;
        Ok(self.control.stop()?)
    }

    /// Whether the song is playing.
    pub fn is_playing(&self) -> bool {
        self.control.is_playing()
    }

    /// Playback position in bars (fractional), while playing.
    pub fn position_bars(&self) -> Option<f64> {
        let positions = self.positions.filter(|_| self.is_playing())?;
        Some(positions.bars_at(self.control.position()))
    }

    /// Playback position in seconds from the start of the song, while playing. Exact: it counts
    /// the samples the audio thread has played (looping jumps back with it).
    #[allow(
        clippy::cast_precision_loss,
        reason = "sample counts stay far below 2^52"
    )]
    pub fn position_seconds(&self) -> Option<f64> {
        self.is_playing()
            .then(|| self.control.position() as f64 / f64::from(self.sample_rate))
    }

    /// Sends the current song to the audio thread if it changed since playback started or was
    /// last synced, and frees memory the audio thread handed back. Returns whether it sent an
    /// update.
    ///
    /// # Errors
    ///
    /// Returns [`PlaybackError`] if the song cannot be compiled or the audio thread does not
    /// respond.
    pub fn sync(&mut self, editor: &Editor) -> Result<bool, PlaybackError> {
        self.control.collect_garbage();
        if !self.is_playing() {
            self.version = None;
            return Ok(false);
        }
        if self.version == Some(version(editor)) {
            return Ok(false);
        }
        let schedule = Schedule::compile(editor.song(), self.sample_rate)?;
        self.positions = Some(schedule.position_map());
        self.control.update(schedule)?;
        self.version = Some(version(editor));
        Ok(true)
    }
}

fn version(editor: &Editor) -> (u64, u64) {
    (editor.id(), editor.revision())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::PatternCommand;
    use crate::{Command, Origin};
    use soundbit_core::{Instrument, Note, Pattern, Song, TrackSettings, Voice};

    fn editor() -> (Editor, soundbit_core::PatternId) {
        let mut song = Song::new("Live");
        let lead = song
            .add_instrument(Instrument::new("Lead", Voice::Sine))
            .expect("room");
        let mut pattern = Pattern::new("A", lead);
        pattern
            .insert_note(Note::new("A4".parse().expect("valid"), 0, 8).expect("valid"))
            .expect("empty");
        let pattern = song.add_pattern(pattern).expect("valid");
        let track = song.add_track(TrackSettings::new("T")).expect("room");
        song.place_clip(track, 0, pattern).expect("valid");
        song.place_clip(track, 1, pattern).expect("valid");
        (Editor::new(song), pattern)
    }

    #[test]
    fn plays_reports_position_and_syncs_edits() {
        let (mut editor, pattern) = editor();
        let (control, mut rt) = player();
        let mut playback = Playback::new(control, 8_000);
        playback.play(&editor, 1, true).expect("starts");
        let mut block = vec![0.0; 2 * 400];
        rt.process(&mut block);
        let bars = playback.position_bars().expect("playing");
        assert!((1.0..1.1).contains(&bars), "position {bars}");
        // Bar 1 at 120 bpm starts at 2 s; 400 frames later is 2.05 s at 8 kHz.
        let seconds = playback.position_seconds().expect("playing");
        assert!((seconds - 2.05).abs() < 1e-9, "position {seconds} s");

        assert!(!playback.sync(&editor).expect("no change"));
        let edit: Command = PatternCommand::Transpose(pattern, 12).into();
        editor.execute(edit, Origin::User).expect("valid");
        assert!(playback.sync(&editor).expect("sends update"));
        rt.process(&mut block);

        playback.stop().expect("stops");
        rt.process(&mut block);
        assert_eq!(playback.position_bars(), None);
        assert_eq!(playback.position_seconds(), None);
    }
}
