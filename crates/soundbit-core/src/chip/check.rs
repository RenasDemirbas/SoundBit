//! Checking a song against its chip profile.

use std::collections::HashSet;

use thiserror::Error;

use super::Channel;
use crate::{Pattern, PatternId, Song, TrackId, VoiceKind};

/// A way in which a song breaks the limits of its chip profile.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ProfileViolation {
    /// The song has more tracks than the profile has channels.
    #[error("the {profile} profile has {max} channels, but the song has {tracks} tracks")]
    TooManyTracks {
        /// Profile name.
        profile: &'static str,
        /// Tracks in the song.
        tracks: usize,
        /// Channels in the profile.
        max: usize,
    },

    /// A pattern on a track uses a voice its channel cannot produce.
    #[error(
        "{track} plays on the {channel} channel, which cannot produce {voice} voices, but \
         {pattern} at bar {bar} uses one"
    )]
    VoiceNotAllowed {
        /// The offending track.
        track: TrackId,
        /// Channel name.
        channel: &'static str,
        /// First bar where the problem occurs.
        bar: u16,
        /// The offending pattern.
        pattern: PatternId,
        /// The voice kind used.
        voice: VoiceKind,
    },

    /// A pattern on a track plays more notes at once than its channel allows.
    #[error(
        "{track} plays on the {channel} channel, which plays {max} note(s) at a time, but \
         {pattern} at bar {bar} plays {needed}"
    )]
    TooManyNotes {
        /// The offending track.
        track: TrackId,
        /// Channel name.
        channel: &'static str,
        /// First bar where the problem occurs.
        bar: u16,
        /// The offending pattern.
        pattern: PatternId,
        /// Notes sounding at once.
        needed: usize,
        /// Notes the channel allows at once.
        max: usize,
    },
}

/// All profile violations of `song`. Each pattern is reported once per track, at the first bar
/// where it appears.
pub(crate) fn violations(song: &Song) -> Vec<ProfileViolation> {
    let profile = song.chip_profile();
    let Some(channels) = profile.channels() else {
        return Vec::new();
    };

    let mut found = Vec::new();
    if song.track_count() > channels.len() {
        found.push(ProfileViolation::TooManyTracks {
            profile: profile.name(),
            tracks: song.track_count(),
            max: channels.len(),
        });
    }

    let mut seen = HashSet::new();
    for ((track_id, track), channel) in song.tracks().zip(channels) {
        for (bar, pattern_id) in track.clips() {
            if !seen.insert((track_id, pattern_id)) {
                continue;
            }
            if let Some(pattern) = song.pattern(pattern_id) {
                let clip = Clip {
                    track: track_id,
                    channel,
                    bar,
                    pattern_id,
                    pattern,
                };
                check_clip(song, &clip, &mut found);
            }
        }
    }
    found
}

struct Clip<'a> {
    track: TrackId,
    channel: &'a Channel,
    bar: u16,
    pattern_id: PatternId,
    pattern: &'a Pattern,
}

fn check_clip(song: &Song, clip: &Clip<'_>, found: &mut Vec<ProfileViolation>) {
    if let Some(instrument) = song.instrument(clip.pattern.instrument) {
        let voice = instrument.voice.kind();
        if !clip.channel.voices.contains(&voice) {
            found.push(ProfileViolation::VoiceNotAllowed {
                track: clip.track,
                channel: clip.channel.name,
                bar: clip.bar,
                pattern: clip.pattern_id,
                voice,
            });
        }
    }

    let needed = clip.pattern.polyphony();
    if needed > clip.channel.polyphony {
        found.push(ProfileViolation::TooManyNotes {
            track: clip.track,
            channel: clip.channel.name,
            bar: clip.bar,
            pattern: clip.pattern_id,
            needed,
            max: clip.channel.polyphony,
        });
    }
}
