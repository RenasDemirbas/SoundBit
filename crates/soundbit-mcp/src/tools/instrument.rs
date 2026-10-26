//! Instrument tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use soundbit_app::command::{InstrumentChanges, InstrumentCommand};
use soundbit_app::model::{Instrument, InstrumentId};
use soundbit_app::{Created, Outcome, PresetLibrary, Session};

use super::{ToolResult, attempt, created, refuse};
use crate::SoundBitServer;
use crate::input::{
    EnvelopeInput, FilterInput, PitchEnvelopeInput, UnisonInput, VibratoInput, VoiceInput, optional,
};
use crate::view::instrument_view;

/// How an instrument is shaped and mixed. Every field is optional; unset fields keep the
/// current value, or the default for a new instrument (sound shaping off, volume 80, pan 0).
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub(crate) struct InstrumentSettingsInput {
    /// Amplitude envelope. New instruments default to a short, plucky envelope.
    #[serde(default)]
    envelope: EnvelopeInput,
    /// Pitch glide at the start of each note.
    #[serde(default)]
    pitch_envelope: PitchEnvelopeInput,
    /// Pitch wobble for held notes.
    #[serde(default)]
    vibrato: VibratoInput,
    /// Detuned copies of the voice.
    #[serde(default)]
    unison: UnisonInput,
    /// Resonant filter.
    #[serde(default)]
    filter: FilterInput,
    /// Volume in percent, 0-100.
    volume: Option<u8>,
    /// Stereo position, -100 (left) to 100 (right).
    pan: Option<i8>,
    /// Share sent to the song's reverb in percent, 0-100; 20-50 places an instrument in a
    /// room, 0 keeps it dry (kicks and basses usually stay dry).
    reverb_send_percent: Option<u8>,
}

impl InstrumentSettingsInput {
    fn to_changes(&self) -> Result<InstrumentChanges, String> {
        Ok(InstrumentChanges {
            envelope: self.envelope.to_changes()?,
            pitch_envelope: self.pitch_envelope.to_changes()?,
            vibrato: self.vibrato.to_changes()?,
            unison: self.unison.to_changes()?,
            filter: self.filter.to_changes()?,
            volume: optional(self.volume)?,
            pan: optional(self.pan)?,
            reverb_send: optional(self.reverb_send_percent)?,
            ..InstrumentChanges::default()
        })
    }
}

/// A new instrument.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct InstrumentAddInput {
    /// Display name, e.g. "Lead" or "Kick".
    name: String,
    /// Sound source.
    voice: VoiceInput,
    #[serde(flatten)]
    settings: InstrumentSettingsInput,
}

/// Changes to an instrument. Only the fields you set are changed.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct InstrumentUpdateInput {
    /// Instrument id.
    id: u32,
    /// New name.
    name: Option<String>,
    /// New sound source (replaces the whole voice).
    voice: Option<VoiceInput>,
    #[serde(flatten)]
    settings: InstrumentSettingsInput,
}

/// A preset to add as a new instrument.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct InstrumentAddPresetInput {
    /// Preset id from `preset_list`, e.g. `strings_ensemble` or `kick`.
    preset: String,
    /// Display name; defaults to the preset's name.
    name: Option<String>,
}

/// An instrument to remove.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct InstrumentIdInput {
    /// Instrument id.
    id: u32,
}

fn new_instrument(input: InstrumentAddInput) -> Result<Instrument, String> {
    let base = Instrument::new(input.name, input.voice.to_voice()?);
    Ok(input.settings.to_changes()?.applied_to(&base))
}

fn changes(input: InstrumentUpdateInput) -> Result<InstrumentChanges, String> {
    Ok(InstrumentChanges {
        name: input.name,
        voice: input.voice.as_ref().map(VoiceInput::to_voice).transpose()?,
        ..input.settings.to_changes()?
    })
}

fn preset_instrument(input: InstrumentAddPresetInput) -> Result<Instrument, String> {
    let library = PresetLibrary::builtin().map_err(|error| error.to_string())?;
    let Some(preset) = library.find(&input.preset) else {
        return Err(format!(
            "unknown preset \"{}\"; call preset_list for the choices: {}",
            input.preset,
            library.ids().join(", ")
        ));
    };
    let mut instrument = preset.instrument.clone();
    if let Some(name) = input.name {
        instrument.name = name;
    }
    Ok(instrument)
}

fn show(id: InstrumentId) -> impl FnOnce(&Session, &Outcome) -> serde_json::Value {
    move |session, _| instrument_view(session.song(), id)
}

fn show_created(session: &Session, outcome: &Outcome) -> serde_json::Value {
    let pick = |c: &Created| {
        if let Created::Instrument(id) = c {
            Some(*id)
        } else {
            None
        }
    };
    created(outcome, pick).map_or(serde_json::Value::Null, |id| {
        instrument_view(session.song(), id)
    })
}

#[tool_router(router = instrument_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Adds an instrument: a voice (pulse, triangle, sawtooth, sine, noise, \
                       wavetable or fm) with envelope, optional sound shaping (pitch envelope, \
                       vibrato, unison, filter), volume, pan and reverb send. Returns its id. \
                       See reference_get for voice parameters; instrument_add_preset is quicker \
                       for common sounds."
    )]
    fn instrument_add(&self, Parameters(input): Parameters<InstrumentAddInput>) -> ToolResult {
        let instrument = attempt!(new_instrument(input));
        self.apply(InstrumentCommand::Add(instrument), show_created)
    }

    #[tool(
        description = "Adds a ready-made instrument from the preset library (strings, brass, \
                       choir, timpani, drums, chip leads, FM keys...). Returns its id and full \
                       settings, which instrument_update can tweak. List presets with \
                       preset_list."
    )]
    fn instrument_add_preset(
        &self,
        Parameters(input): Parameters<InstrumentAddPresetInput>,
    ) -> ToolResult {
        let instrument = attempt!(preset_instrument(input));
        self.apply(InstrumentCommand::Add(instrument), show_created)
    }

    #[tool(
        description = "Changes an instrument's name, voice, envelope, sound shaping, volume, pan \
                       or reverb send. Only the given fields change, also inside groups such as \
                       vibrato or filter."
    )]
    fn instrument_update(
        &self,
        Parameters(input): Parameters<InstrumentUpdateInput>,
    ) -> ToolResult {
        let id = InstrumentId::new(input.id);
        let changes = attempt!(changes(input));
        if changes == InstrumentChanges::default() {
            return refuse("nothing to change: set at least one field besides `id`");
        }
        self.apply(InstrumentCommand::Update(id, changes), show(id))
    }

    #[tool(
        description = "Removes an instrument. Fails while a pattern still uses it; change or \
                       remove those patterns first.",
        annotations(destructive_hint = true)
    )]
    fn instrument_remove(&self, Parameters(input): Parameters<InstrumentIdInput>) -> ToolResult {
        let id = InstrumentId::new(input.id);
        self.apply(
            InstrumentCommand::Remove(id),
            move |_, _| serde_json::json!({"removed_instrument_id": id.get()}),
        )
    }
}
