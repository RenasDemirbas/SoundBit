//! Guidance for agents: server instructions and the `reference_get` content.

use serde_json::{Value, json};
use soundbit_app::model::limits::{
    MAX_BARS, MAX_INSTRUMENTS, MAX_NOTES_PER_PATTERN, MAX_PATTERNS, MAX_TRACKS,
};
use soundbit_app::model::{ChipProfile, ScaleKind};

/// Sent to the client at connection time.
pub(crate) const INSTRUCTIONS: &str = "\
SoundBit composes 8-bit and 16-bit chiptune music. One song is open at a time; every change \
you make is undoable (history_undo) and is shown to the user as an 'AI:' step.

Song model:
- Instruments define a sound: a voice (pulse, triangle, sawtooth, sine, noise, wavetable, fm) \
plus envelope, volume and pan, and optional sound shaping: a pitch envelope (kicks, toms, \
timpani, lasers), vibrato, unison (detuned copies for strings and choirs), a resonant filter \
and a reverb send into the song's shared reverb room. preset_list offers ready-made \
instruments (orchestral strings, brass, choir, timpani, drum kit, chip leads, FM keys); \
instrument_add_preset adds one in a single call.
- A pattern is exactly ONE BAR of notes for one instrument. Steps are numbered from 0; a bar has \
steps_per_bar steps (default 16 = sixteenth notes in 4/4). Notes are written as names with an \
octave: C4 is middle C, F#3, Bb5. Length is in steps.
- Tracks are lanes. Placing a pattern on a track at a bar creates a clip; each bar of a track \
holds at most one pattern. Reuse patterns across bars for repetition.
- An optional chip profile (nes, game_boy, c64, genesis) limits tracks, voices per track and \
notes at once (1 per track). Changes that break the profile are rejected with an explanation.

Workflow: reference_get (once) -> song_get or song_new -> preset_list + instrument_add_preset \
(or instrument_add) -> pattern_create -> track_add -> arrangement_place_many for the whole \
arrangement -> song_get to review -> export_wav to render audio and check the per-track levels \
-> song_save. Paths are relative to the workspace. Errors explain what to fix; read them and \
retry.";

/// Reference data about voices, profiles, scales and limits.
pub(crate) fn reference() -> Value {
    json!({
        "notes": {
            "format": "Letter A-G, optional # or b, octave number: C4 (middle C, MIDI 60), F#3, Bb5. Range C-1 to G9.",
            "timing": "start and length are in steps; a bar has steps_per_bar steps (steps_per_beat x beats_per_bar).",
            "velocity": "1-127, default 100.",
        },
        "voices": {
            "pulse": {"params": {"duty": ["eighth", "quarter", "half", "three_quarters"]}, "use": "NES/Game Boy leads and harmonies; quarter is the classic lead."},
            "triangle": {"params": {}, "use": "NES-style stepped triangle; basslines."},
            "sawtooth": {"params": {}, "use": "Bright, buzzy leads and pads (C64 style)."},
            "sine": {"params": {}, "use": "Pure tone; soft pads, sub bass."},
            "noise": {"params": {"mode": ["long", "short"]}, "use": "Drums. Long = hiss (snare, hats); short = metallic. Higher notes sound brighter: kick ~C3, snare ~A5, hats ~C8."},
            "wavetable": {"params": {"samples": "exactly 32 integers 0-15 (one waveform cycle)"}, "use": "Game Boy wave channel: custom timbres."},
            "fm": {
                "params": {
                    "algorithm": "0-7 operator routing (0 = 1>2>3>4 serial, 4 = two pairs, 7 = four parallel sines)",
                    "feedback": "0-7 self-modulation of operator 1",
                    "operators": "up to 4 objects: multiple 0-15, detune -3..3, total_level 0-127 (0 loudest), attack_rate/decay_rate/sustain_rate 0-31, sustain_level 0-15, release_rate 0-15",
                },
                "use": "Sega Genesis style: basses, bells, brass. Ignores the instrument envelope.",
            },
        },
        "envelope": "attack_ms, decay_ms, release_ms 0-10000; sustain_percent 0-100.",
        "sound_shaping": {
            "pitch_envelope": "semitones -48..48 (start offset, glides to the note) + time_ms. Kick: sine, +36 over 50 ms, play C2. Tom: +12 over 120 ms. Timpani: +5 over 90 ms.",
            "vibrato": "depth_cents 0-100, rate_hz 0.1-20, delay_ms. Strings/flute: 10-20 cents, 5-6 Hz, 150-300 ms delay.",
            "unison": "voices 1-4, detune_cents 0-50 (outermost copies). Strings: 3 voices, 12-15 cents. Supersaw: 4 voices, 25 cents. Noise ignores it.",
            "filter": "mode off/low_pass/high_pass/band_pass, cutoff_hz 20-20000, resonance_percent 0-100, envelope_semitones -72..72 (cutoff follows the amplitude envelope). Soft strings: saw + low_pass 3000. Acid bass: low_pass 300, resonance 75, envelope 40.",
            "reverb": "reverb_send_percent 0-100 per instrument into one shared room; song_update reverb_size and reverb_damping shape the room. Orchestral: room 60-80, sends 25-50; keep kicks and basses dry.",
        },
        "presets": "Call preset_list for ready-made instruments; their descriptions say which octaves or drum notes to use.",
        "chip_profiles": chip_profiles(),
        "scales": ScaleKind::ALL.iter().map(|kind| json!({"kind": kind, "intervals": kind.intervals()})).collect::<Vec<_>>(),
        "limits": {
            "instruments": MAX_INSTRUMENTS,
            "patterns": MAX_PATTERNS,
            "tracks": MAX_TRACKS,
            "notes_per_pattern": MAX_NOTES_PER_PATTERN,
            "bars": MAX_BARS,
            "bpm": "20-400",
            "steps_per_beat": "1-16",
            "beats_per_bar": "1-16",
        },
    })
}

fn chip_profiles() -> Vec<Value> {
    ChipProfile::ALL
        .iter()
        .map(|profile| {
            let channels = profile.channels().map(|channels| {
                channels
                    .iter()
                    .enumerate()
                    .map(|(index, channel)| {
                        json!({
                            "track": index + 1,
                            "name": channel.name,
                            "voices": channel.voices,
                            "notes_at_once": channel.polyphony,
                        })
                    })
                    .collect::<Vec<_>>()
            });
            json!({"profile": profile, "name": profile.name(), "channels": channels})
        })
        .collect()
}
