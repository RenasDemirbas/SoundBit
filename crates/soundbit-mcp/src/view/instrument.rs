//! Instrument views.

use serde_json::{Value, json};
use soundbit_app::model::{InstrumentId, Song};

/// Full details of an instrument, or `null` if it does not exist.
pub(crate) fn instrument_view(song: &Song, id: InstrumentId) -> Value {
    let Some(instrument) = song.instrument(id) else {
        return Value::Null;
    };
    let used_by: Vec<u32> = song
        .patterns()
        .filter(|(_, pattern)| pattern.instrument == id)
        .map(|(pattern_id, _)| pattern_id.get())
        .collect();
    let envelope = instrument.envelope;
    let sweep = instrument.pitch_envelope;
    let vibrato = instrument.vibrato;
    let unison = instrument.unison;
    let filter = instrument.filter;
    json!({
        "id": id.get(),
        "name": instrument.name,
        "voice": instrument.voice,
        "envelope": {
            "attack_ms": envelope.attack.get(),
            "decay_ms": envelope.decay.get(),
            "sustain_percent": envelope.sustain.get(),
            "release_ms": envelope.release.get(),
        },
        "pitch_envelope": {"semitones": sweep.semitones.get(), "time_ms": sweep.time.get()},
        "vibrato": {
            "depth_cents": vibrato.depth.get(),
            "rate_hz": f64::from(vibrato.rate.get()) / 10.0,
            "delay_ms": vibrato.delay.get(),
        },
        "unison": {"voices": unison.voices.get(), "detune_cents": unison.detune.get()},
        "filter": {
            "mode": filter.mode,
            "cutoff_hz": filter.cutoff.get(),
            "resonance_percent": filter.resonance.get(),
            "envelope_semitones": filter.envelope_depth.get(),
        },
        "volume": instrument.volume.get(),
        "pan": instrument.pan.get(),
        "reverb_send_percent": instrument.reverb_send.get(),
        "used_by_patterns": used_by,
    })
}
