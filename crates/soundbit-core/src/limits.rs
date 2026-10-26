//! Song-wide size limits.
//!
//! They keep songs within what the engine can play in real time and stop runaway edits
//! (for example from an AI agent stuck in a loop).

/// Maximum number of instruments in a song.
pub const MAX_INSTRUMENTS: usize = 64;

/// Maximum number of patterns in a song.
pub const MAX_PATTERNS: usize = 1024;

/// Maximum number of tracks in a song.
pub const MAX_TRACKS: usize = 16;

/// Maximum number of notes in one pattern.
pub const MAX_NOTES_PER_PATTERN: usize = 512;

/// Maximum arrangement length in bars.
pub const MAX_BARS: u16 = 512;
