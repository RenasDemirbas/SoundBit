//! Bounded integer newtypes for musical and sound-design parameters.

/// Defines an integer newtype that can only hold values in an inclusive range.
///
/// The generated type validates on construction and on deserialization, so an out-of-range
/// value can never exist inside a song. The default is checked against the range at compile
/// time.
macro_rules! ranged {
    (
        $(#[$meta:meta])*
        $name:ident($repr:ty): $min:literal..=$max:literal, default $default:literal, $label:literal
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name($repr);

        const _: () = assert!($min <= $default && $default <= $max, "default out of range");

        impl $name {
            /// Smallest allowed value.
            pub const MIN: Self = Self($min);
            /// Largest allowed value.
            pub const MAX: Self = Self($max);

            /// Creates a value, rejecting anything outside the allowed range.
            ///
            /// # Errors
            ///
            /// Returns [`ModelError::OutOfRange`](crate::ModelError::OutOfRange) if `value` is
            /// outside the range.
            pub fn new(value: $repr) -> Result<Self, $crate::ModelError> {
                if (Self::MIN.0..=Self::MAX.0).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err($crate::ModelError::OutOfRange {
                        name: $label,
                        value: i64::from(value),
                        min: i64::from(Self::MIN.0),
                        max: i64::from(Self::MAX.0),
                    })
                }
            }

            /// Creates a value, clamping anything outside the allowed range.
            pub const fn saturating(value: $repr) -> Self {
                if value < Self::MIN.0 {
                    Self::MIN
                } else if value > Self::MAX.0 {
                    Self::MAX
                } else {
                    Self(value)
                }
            }

            /// Returns the raw value.
            pub const fn get(self) -> $repr {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        impl TryFrom<$repr> for $name {
            type Error = $crate::ModelError;

            fn try_from(value: $repr) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for $repr {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <$repr as serde::Deserialize>::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use ranged;
