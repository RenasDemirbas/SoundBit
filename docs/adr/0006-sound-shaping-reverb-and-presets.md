# 0006. Sound shaping, reverb and data-driven presets

- Status: Accepted
- Date: 2026-10-26

## Context

Composing an orchestral-style battle theme through MCP showed what the bare voices cannot do.
A kick drum needs a falling pitch; a noise burst does not punch. Strings and choirs need
detuned layers and a softened tone; one sawtooth is thin and harsh. Held notes need vibrato.
Everything sounded dry, while orchestral and 16-bit soundtracks rely on room ambience. Agents
also had to guess FM parameters for common sounds, and had no way to hear their mix.

The real-time and determinism rules (ADR 0003) apply to everything new: no allocation on the
audio thread, no platform math functions, and bit-exact output.

## Decision

**Instrument sound shaping (`soundbit-core`).** `Instrument` gains five optional groups, all
validated `ranged!` newtypes that default to "off":

| Field            | Meaning                                                           |
|------------------|-------------------------------------------------------------------|
| `pitch_envelope` | start offset in semitones (±48) gliding to the note over `time` ms |
| `vibrato`        | depth in cents (0–100), rate in 0.1 Hz (0.1–20 Hz), delay in ms     |
| `unison`         | 1–4 copies, outermost detuned by 0–50 cents                       |
| `filter`         | off / low-pass / high-pass / band-pass, cutoff, resonance, envelope depth in semitones |
| `reverb_send`    | percent of the instrument sent to the song's reverb               |

Songs gain one shared `reverb` room (size, damping). One shared room, rather than an effect
per track, keeps the cost fixed and mirrors how soundtrack mixes place instruments in one space.

**DSP (`soundbit-dsp`).** New `no_std` building blocks: `math::exp2` (exponent bits plus a
Taylor series, so only `+ - * /` and conversions), a trapezoidal state-variable filter whose
`tan` is computed from `math::sine`, a pitch modulator, and a Freeverb-structured reverb
(8 combs + 4 allpasses per channel). The reverb cannot allocate, so it works on a buffer the
caller owns.

**Engine.** Each voice holds up to four unison layers inline (`[Source; 4]`), a pitch
modulator and a filter, so starting a note still never allocates. `Engine::new` allocates the
reverb's delay lines once, sized for the sample rate; schedule swaps only update its
parameters. The reverb is skipped entirely when no instrument sends to it and its tail has
died away, which keeps songs without sends bit-identical to before (`SHOWCASE_HASH` is
unchanged). A second golden hash, `SHAPED_HASH`, pins the new features. Offline rendering can
meter each track (peak and mean square) for mix reports.

**Project format version 2.** The new fields are optional, so version 1 files load unchanged;
`migrations/v1_to_v2.rs` is an explicit no-op step. The version still changes because older
builds would silently drop the new settings when re-saving.

**Presets as data.** Instrument presets live in `assets/presets/*.json`
(`"format": "soundbit-presets"`, with a `source` field per file). `soundbit-formats` parses
them; `soundbit-app::PresetLibrary` embeds the files with `include_str!` so every front end
has the same library without locating files at run time. A test renders every preset and
fails on silence or clipping. The desktop app's hard-coded TypeScript presets are removed.

**MCP.** New tools `preset_list`, `instrument_add_preset` and `arrangement_place_many`;
`instrument_add`/`instrument_update` accept the shaping groups (partial updates inside each
group); `song_update` accepts `reverb_size`/`reverb_damping`; `export_wav` reports the RMS of
the mix and peak/RMS per track so agents can balance a mix they cannot hear. All schema
changes are additive.

## Consequences

- Chip profiles do not restrict the new features: a NES-profile song may use reverb or
  unison, which the hardware could not. Profiles stay *soft* constraint sets (ADR 0002);
  restricting effects per profile is possible later as additional violations.
- Each voice is larger (four inline sources). The engine preallocates 16 × 8 voices, which is
  a few hundred kilobytes; CPU cost grows only for notes that use unison or a filter.
- The filter's cutoff follows the amplitude envelope; there is no separate filter envelope.
  A dedicated envelope, LFOs and per-note effects (slide, arpeggio) are natural next steps.
- `include_str!` reaches outside the crate directory, which is fine for this unpublished
  workspace but would need a build script if `soundbit-app` were published.
