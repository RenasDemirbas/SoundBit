//! String choices backed by domain enums.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use soundbit_app::model::{ChipProfile, DutyCycle, FilterMode, NoiseMode, ScaleKind};

/// A domain enum whose variants an agent picks by name.
pub(crate) trait Choices: Copy + Serialize + DeserializeOwned + 'static {
    /// Schema name.
    const NAME: &'static str;
    /// Every variant, serialized as the names agents use.
    fn all() -> &'static [Self];
}

/// Accepts a domain enum by its serialized name. The schema lists the valid names, generated
/// from the enum itself, so it can never drift from the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub(crate) struct Choice<T>(pub(crate) T);

impl<T: Choices> JsonSchema for Choice<T> {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed(T::NAME)
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        let names: Vec<_> = T::all()
            .iter()
            .filter_map(|choice| serde_json::to_value(choice).ok())
            .collect();
        json_schema!({ "type": "string", "enum": names })
    }
}

impl Choices for ScaleKind {
    const NAME: &'static str = "ScaleKind";
    fn all() -> &'static [Self] {
        &Self::ALL
    }
}

impl Choices for ChipProfile {
    const NAME: &'static str = "ChipProfile";
    fn all() -> &'static [Self] {
        &Self::ALL
    }
}

impl Choices for DutyCycle {
    const NAME: &'static str = "DutyCycle";
    fn all() -> &'static [Self] {
        &[Self::Eighth, Self::Quarter, Self::Half, Self::ThreeQuarters]
    }
}

impl Choices for NoiseMode {
    const NAME: &'static str = "NoiseMode";
    fn all() -> &'static [Self] {
        &[Self::Long, Self::Short]
    }
}

impl Choices for FilterMode {
    const NAME: &'static str = "FilterMode";
    fn all() -> &'static [Self] {
        &[Self::Off, Self::LowPass, Self::HighPass, Self::BandPass]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_lists_serialized_names() {
        let schema = schemars::schema_for!(Choice<ChipProfile>);
        let names = schema.get("enum").expect("has enum");
        assert_eq!(
            names,
            &serde_json::json!(["free", "nes", "game_boy", "c64", "genesis"])
        );
    }

    #[test]
    fn parses_names() {
        let duty: Choice<DutyCycle> = serde_json::from_str("\"three_quarters\"").expect("valid");
        assert_eq!(duty.0, DutyCycle::ThreeQuarters);
        assert!(serde_json::from_str::<Choice<DutyCycle>>("\"third\"").is_err());
    }
}
