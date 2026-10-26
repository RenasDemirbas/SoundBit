//! Sound-shaping settings: pitch envelope, vibrato, unison and filter.
//!
//! Every field is optional. Unset fields keep the instrument's current value (or the default,
//! which is "off", for a new instrument).

use schemars::JsonSchema;
use serde::Deserialize;
use soundbit_app::command::{FilterChanges, PitchEnvelopeChanges, UnisonChanges, VibratoChanges};
use soundbit_app::model::{FilterMode, VibratoRate};

use super::{Choice, optional};

/// Tenths of a hertz per hertz: vibrato rates are stored in 0.1 Hz.
const RATE_SCALE: f32 = 10.0;

/// A pitch glide at the start of every note: the note starts `semitones` away and slides to
/// its own pitch over `time_ms`. Use it for kicks and toms (+24 to +36 over 40-120 ms),
/// timpani (+3 to +7), lasers and risers.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct PitchEnvelopeInput {
    /// Start offset in semitones, -48 to 48; positive values fall into the note. 0 turns the
    /// glide off.
    semitones: Option<i8>,
    /// Glide time in milliseconds, 0-10000.
    time_ms: Option<u16>,
}

impl PitchEnvelopeInput {
    pub(crate) fn to_changes(&self) -> Result<PitchEnvelopeChanges, String> {
        Ok(PitchEnvelopeChanges {
            semitones: optional(self.semitones)?,
            time: optional(self.time_ms)?,
        })
    }
}

/// A periodic pitch wobble for held notes (strings, flutes, leads).
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct VibratoInput {
    /// Depth in cents either side of the note, 0-100; 10-25 sounds natural. 0 turns vibrato
    /// off.
    depth_cents: Option<u8>,
    /// Speed in hertz, 0.1-20.0; 5-6 sounds natural.
    rate_hz: Option<f32>,
    /// Milliseconds before the vibrato starts, 0-10000; 150-300 sounds like a real player.
    delay_ms: Option<u16>,
}

impl VibratoInput {
    pub(crate) fn to_changes(&self) -> Result<VibratoChanges, String> {
        Ok(VibratoChanges {
            depth: optional(self.depth_cents)?,
            rate: self.rate_hz.map(rate).transpose()?,
            delay: optional(self.delay_ms)?,
        })
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is range-checked before the cast"
)]
fn rate(hz: f32) -> Result<VibratoRate, String> {
    let tenths = (hz * RATE_SCALE).round();
    let (min, max) = (VibratoRate::MIN.get(), VibratoRate::MAX.get());
    if tenths.is_finite() && (f32::from(min)..=f32::from(max)).contains(&tenths) {
        VibratoRate::new(tenths as u8).map_err(|error| error.to_string())
    } else {
        Err(format!(
            "vibrato rate_hz must be between {} and {}, got {hz}",
            f32::from(min) / RATE_SCALE,
            f32::from(max) / RATE_SCALE
        ))
    }
}

/// Several detuned copies of the voice playing together: turns a thin wave into an ensemble
/// (strings, choirs, supersaws). Noise voices ignore it.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct UnisonInput {
    /// Number of copies, 1-4. 1 turns unison off.
    voices: Option<u8>,
    /// Detune of the outermost copies in cents either side, 0-50; 8-20 is lush, 25+ is wide.
    detune_cents: Option<u8>,
}

impl UnisonInput {
    pub(crate) fn to_changes(&self) -> Result<UnisonChanges, String> {
        Ok(UnisonChanges {
            voices: optional(self.voices)?,
            detune: optional(self.detune_cents)?,
        })
    }
}

/// A resonant filter on every note. A low-pass softens bright waves (saw, pulse) into strings,
/// pads and basses; an envelope depth makes the tone open as each note starts.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub(crate) struct FilterInput {
    /// Filter type; "off" bypasses the filter.
    mode: Option<Choice<FilterMode>>,
    /// Cutoff frequency in hertz, 20-20000.
    cutoff_hz: Option<u16>,
    /// Emphasis around the cutoff in percent, 0-100; above 70 it whistles.
    resonance_percent: Option<u8>,
    /// How far the amplitude envelope moves the cutoff, in semitones at full level, -72 to 72.
    envelope_semitones: Option<i8>,
}

impl FilterInput {
    pub(crate) fn to_changes(&self) -> Result<FilterChanges, String> {
        Ok(FilterChanges {
            mode: self.mode.map(|mode| mode.0),
            cutoff: optional(self.cutoff_hz)?,
            resonance: optional(self.resonance_percent)?,
            envelope_depth: optional(self.envelope_semitones)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vibrato_rate_is_converted_and_checked() {
        let input: VibratoInput = serde_json::from_str(r#"{"rate_hz": 5.5}"#).expect("valid");
        let changes = input.to_changes().expect("in range");
        assert_eq!(changes.rate.map(VibratoRate::get), Some(55));
        let input: VibratoInput = serde_json::from_str(r#"{"rate_hz": 40}"#).expect("valid");
        let error = input.to_changes().expect_err("too fast");
        assert_eq!(error, "vibrato rate_hz must be between 0.1 and 20, got 40");
    }

    #[test]
    fn filter_errors_name_the_range() {
        let input: FilterInput = serde_json::from_str(r#"{"cutoff_hz": 5}"#).expect("valid");
        let error = input.to_changes().expect_err("too low");
        assert!(error.contains("between 20 and 20000"), "{error}");
    }
}
