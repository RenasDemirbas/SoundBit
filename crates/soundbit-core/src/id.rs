//! Opaque identifiers for song entities.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Common behaviour of entity identifiers, used by [`Collection`](crate::collection::Collection).
pub(crate) trait EntityId: Copy + Eq + fmt::Display {
    fn from_raw(raw: u32) -> Self;
    fn raw(self) -> u32;
}

macro_rules! entity_id {
    ($(#[$meta:meta])* $name:ident, $label:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            /// Wraps a raw id, e.g. one received from an adapter. Whether the entity exists is
            /// checked by [`Song`](crate::Song) when the id is used.
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            /// Returns the raw id.
            pub const fn get(self) -> u32 {
                self.0
            }
        }

        impl EntityId for $name {
            fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            fn raw(self) -> u32 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($label, " {}"), self.0)
            }
        }
    };
}

entity_id!(
    /// Identifies an instrument within a song.
    InstrumentId,
    "instrument"
);

entity_id!(
    /// Identifies a pattern within a song.
    PatternId,
    "pattern"
);

entity_id!(
    /// Identifies a track (arrangement lane) within a song.
    TrackId,
    "track"
);
