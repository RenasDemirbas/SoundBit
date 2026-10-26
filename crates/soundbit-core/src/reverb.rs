//! The song's shared reverb: one room that instruments send part of their sound into.

use serde::{Deserialize, Serialize};

use crate::ranged::ranged;

ranged! {
    /// Size of the reverb room in percent: small rooms decay fast, large halls ring long.
    RoomSize(u8): 0..=100, default 60, "reverb room size"
}

ranged! {
    /// How quickly high frequencies die away in the reverb, in percent. High values sound
    /// like a soft, furnished room; low values like bare stone.
    ReverbDamping(u8): 0..=100, default 40, "reverb damping"
}

ranged! {
    /// How much of an instrument goes into the reverb, in percent. 0 keeps it dry.
    ReverbSend(u8): 0..=100, default 0, "reverb send"
}

/// The room every instrument's reverb send plays into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Reverb {
    /// Room size.
    pub size: RoomSize,
    /// High-frequency damping.
    pub damping: ReverbDamping,
}
