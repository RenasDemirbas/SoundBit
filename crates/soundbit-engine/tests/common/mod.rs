//! Shared fixtures for engine integration tests.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of the helpers"
)]
#![allow(
    clippy::expect_used,
    reason = "fixtures panic on invalid test data by design"
)]

use soundbit_core::{
    Bpm, DutyCycle, EnvelopeTime, Filter, FilterCutoff, FilterEnvelopeDepth, FilterMode,
    FilterResonance, FmMultiple, FmOperator, FmTotalLevel, FmVoice, Instrument, NoiseMode, Note,
    Pattern, PitchEnvelope, PitchSweep, Reverb, ReverbSend, RoomSize, Song, Swing, TimeSettings,
    TrackSettings, Unison, UnisonDetune, UnisonVoices, Vibrato, VibratoDepth, VibratoRate, Voice,
    Wavetable,
};

fn note(name: &str, start: u16, length: u16) -> Note {
    Note::new(name.parse().expect("valid note name"), start, length).expect("non-zero length")
}

/// Adds an instrument, a pattern with `notes`, and a track playing it on bars 0 and 1.
fn add_part(song: &mut Song, name: &str, voice: Voice, notes: &[Note]) {
    add_instrument_part(song, Instrument::new(name, voice), notes);
}

fn add_instrument_part(song: &mut Song, instrument: Instrument, notes: &[Note]) {
    let name = instrument.name.clone();
    let name = name.as_str();
    let instrument = song.add_instrument(instrument).expect("room");
    let mut pattern = Pattern::new(name, instrument);
    pattern.set_notes(notes.to_vec()).expect("valid notes");
    let pattern = song.add_pattern(pattern).expect("valid pattern");
    let track = song.add_track(TrackSettings::new(name)).expect("room");
    for bar in 0..2 {
        song.place_clip(track, bar, pattern).expect("valid clip");
    }
}

fn bell() -> FmVoice {
    let modulator = |multiple: u8, level: u8| FmOperator {
        multiple: FmMultiple::new(multiple).expect("in range"),
        total_level: FmTotalLevel::new(level).expect("in range"),
        ..FmOperator::default()
    };
    FmVoice {
        algorithm: soundbit_core::FmAlgorithm::new(4).expect("in range"),
        operators: [
            modulator(3, 24),
            modulator(1, 0),
            modulator(7, 40),
            modulator(2, 6),
        ],
        ..FmVoice::default()
    }
}

/// Two bars at 150 bpm with swing, using every voice kind: pulse, triangle, noise (both
/// modes), wavetable, FM, and a sawtooth + sine chord.
pub(crate) fn showcase_song() -> Song {
    let mut song = Song::new("Showcase");
    song.set_timing(TimeSettings {
        bpm: Bpm::new(150).expect("in range"),
        swing: Swing::new(20).expect("in range"),
        ..TimeSettings::default()
    })
    .expect("empty song accepts any timing");

    let lead = Voice::Pulse {
        duty: DutyCycle::Quarter,
    };
    add_part(
        &mut song,
        "Lead",
        lead,
        &[
            note("C5", 0, 2),
            note("E5", 4, 2),
            note("G5", 8, 2),
            note("C6", 12, 4),
        ],
    );
    add_part(
        &mut song,
        "Bass",
        Voice::Triangle,
        &[note("C3", 0, 6), note("G2", 8, 6)],
    );
    add_part(
        &mut song,
        "Kick",
        Voice::Noise {
            mode: NoiseMode::Long,
        },
        &[note("C4", 0, 1), note("C4", 8, 1)],
    );
    add_part(
        &mut song,
        "Hat",
        Voice::Noise {
            mode: NoiseMode::Short,
        },
        &[note("C8", 4, 1), note("C8", 12, 1)],
    );
    add_part(
        &mut song,
        "Wave",
        Voice::Wavetable(Wavetable::triangle()),
        &[note("E4", 0, 8)],
    );
    add_part(
        &mut song,
        "Bell",
        Voice::Fm(bell()),
        &[note("G5", 2, 2), note("C6", 10, 2)],
    );
    add_part(&mut song, "Saw", Voice::Sawtooth, &[note("C4", 0, 16)]);
    add_part(&mut song, "Sine", Voice::Sine, &[note("G4", 0, 16)]);
    song
}

fn ms(value: u16) -> EnvelopeTime {
    EnvelopeTime::new(value).expect("in range")
}

/// Two bars using every sound-shaping feature: a pitch-swept sine kick, a vibrato lead, a
/// unison saw pad through a swept resonant low-pass, an FM bell with unison, and reverb.
pub(crate) fn shaped_song() -> Song {
    let mut song = Song::new("Shaped");
    song.set_reverb(Reverb {
        size: RoomSize::new(80).expect("in range"),
        ..Reverb::default()
    });
    let send = |percent| ReverbSend::new(percent).expect("in range");

    let mut kick = Instrument::new("Kick", Voice::Sine);
    kick.pitch_envelope = PitchEnvelope {
        semitones: PitchSweep::new(36).expect("in range"),
        time: ms(60),
    };
    add_instrument_part(&mut song, kick, &[note("C2", 0, 2), note("C2", 8, 2)]);

    let mut lead = Instrument::new(
        "Lead",
        Voice::Pulse {
            duty: DutyCycle::Quarter,
        },
    );
    lead.vibrato = Vibrato {
        depth: VibratoDepth::new(40).expect("in range"),
        rate: VibratoRate::new(60).expect("in range"),
        delay: ms(100),
    };
    lead.reverb_send = send(40);
    add_instrument_part(&mut song, lead, &[note("E5", 0, 8), note("G5", 8, 8)]);

    let mut pad = Instrument::new("Pad", Voice::Sawtooth);
    pad.unison = Unison {
        voices: UnisonVoices::new(3).expect("in range"),
        detune: UnisonDetune::new(20).expect("in range"),
    };
    pad.filter = Filter {
        mode: FilterMode::LowPass,
        cutoff: FilterCutoff::new(400).expect("in range"),
        resonance: FilterResonance::new(60).expect("in range"),
        envelope_depth: FilterEnvelopeDepth::new(36).expect("in range"),
    };
    pad.reverb_send = send(70);
    add_instrument_part(&mut song, pad, &[note("C4", 0, 16), note("G4", 0, 16)]);

    let mut bell = Instrument::new("Bell", Voice::Fm(bell()));
    bell.unison = Unison {
        voices: UnisonVoices::new(2).expect("in range"),
        ..Unison::default()
    };
    bell.filter = Filter {
        mode: FilterMode::HighPass,
        cutoff: FilterCutoff::new(300).expect("in range"),
        ..Filter::default()
    };
    bell.reverb_send = send(100);
    add_instrument_part(&mut song, bell, &[note("C6", 4, 2), note("G5", 12, 2)]);
    song
}

/// FNV-1a hash of the samples quantized to 16 bits, exactly as a WAV export would store them.
pub(crate) fn audio_hash(samples: &[f32]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    samples
        .iter()
        .flat_map(|sample| quantize(*sample).to_le_bytes())
        .fold(OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the value is clamped to the i16 range first"
)]
fn quantize(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
}
