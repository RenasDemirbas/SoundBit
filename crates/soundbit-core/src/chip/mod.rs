//! Chip profiles: optional hardware-style constraints for a song.
//!
//! A profile is a constraint set, not an emulator. It limits how many tracks a song has, which
//! voice kinds each track may use and how many notes a track may play at once. Profiles are
//! built from publicly documented hardware facts and are domain rules, so they live in code.

mod check;

use serde::{Deserialize, Serialize};

pub use check::ProfileViolation;
pub(crate) use check::violations;

use crate::VoiceKind;

/// One hardware channel of a chip profile. Track *n* of the song plays on channel *n*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel {
    /// Hardware channel name, e.g. "Pulse 1".
    pub name: &'static str,
    /// Voice kinds this channel can produce.
    pub voices: &'static [VoiceKind],
    /// Notes the channel can play at the same time.
    pub polyphony: usize,
}

/// A hardware style a song can be constrained to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ChipProfile {
    /// No constraints beyond the song-wide limits.
    #[default]
    Free,
    /// Nintendo Entertainment System (2A03): two pulses, triangle, noise.
    Nes,
    /// Nintendo Game Boy: two pulses, wave channel, noise.
    GameBoy,
    /// Commodore 64 (SID): three voices with selectable waveforms.
    C64,
    /// Sega Genesis / Mega Drive: six FM channels plus the SN76489 PSG.
    Genesis,
}

const fn mono(name: &'static str, voices: &'static [VoiceKind]) -> Channel {
    Channel {
        name,
        voices,
        polyphony: 1,
    }
}

const PULSE: &[VoiceKind] = &[VoiceKind::Pulse];
const TRIANGLE: &[VoiceKind] = &[VoiceKind::Triangle];
const NOISE: &[VoiceKind] = &[VoiceKind::Noise];
const WAVE: &[VoiceKind] = &[VoiceKind::Wavetable];
const FM: &[VoiceKind] = &[VoiceKind::Fm];
const SID: &[VoiceKind] = &[
    VoiceKind::Pulse,
    VoiceKind::Triangle,
    VoiceKind::Sawtooth,
    VoiceKind::Noise,
];

const NES: &[Channel] = &[
    mono("Pulse 1", PULSE),
    mono("Pulse 2", PULSE),
    mono("Triangle", TRIANGLE),
    mono("Noise", NOISE),
];

const GAME_BOY: &[Channel] = &[
    mono("Pulse 1", PULSE),
    mono("Pulse 2", PULSE),
    mono("Wave", WAVE),
    mono("Noise", NOISE),
];

const C64: &[Channel] = &[
    mono("Voice 1", SID),
    mono("Voice 2", SID),
    mono("Voice 3", SID),
];

const GENESIS: &[Channel] = &[
    mono("FM 1", FM),
    mono("FM 2", FM),
    mono("FM 3", FM),
    mono("FM 4", FM),
    mono("FM 5", FM),
    mono("FM 6", FM),
    mono("PSG 1", PULSE),
    mono("PSG 2", PULSE),
    mono("PSG 3", PULSE),
    mono("PSG Noise", NOISE),
];

impl ChipProfile {
    /// Every profile, in display order.
    pub const ALL: [Self; 5] = [
        Self::Free,
        Self::Nes,
        Self::GameBoy,
        Self::C64,
        Self::Genesis,
    ];

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Nes => "NES",
            Self::GameBoy => "Game Boy",
            Self::C64 => "Commodore 64",
            Self::Genesis => "Genesis / Mega Drive",
        }
    }

    /// The channels of this profile, or `None` if it is unconstrained.
    pub const fn channels(self) -> Option<&'static [Channel]> {
        match self {
            Self::Free => None,
            Self::Nes => Some(NES),
            Self::GameBoy => Some(GAME_BOY),
            Self::C64 => Some(C64),
            Self::Genesis => Some(GENESIS),
        }
    }
}
