//! Version 2 adds sound shaping.
//!
//! Instruments gain `pitch_envelope`, `vibrato`, `unison`, `filter` and `reverb_send`, and
//! songs gain a `reverb` room. Every new field is optional and defaults to "off", so a
//! version 1 song loads unchanged and sounds exactly as before. Older builds cannot read
//! version 2 files, which is why the version changed at all: they would silently drop the new
//! settings.

use serde_json::Value;

/// Nothing to rewrite: the missing fields take their defaults when the song is read.
pub(super) fn migrate(_song: &mut Value) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_the_song_untouched() {
        let mut song = serde_json::json!({"instruments": [{"id": 1, "voice": {"type": "sine"}}]});
        let before = song.clone();
        migrate(&mut song);
        assert_eq!(song, before);
    }
}
