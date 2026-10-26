# 0003. Deterministic, real-time safe audio

- Status: Accepted
- Date: 2026-10-25

## Context

Audio regressions are hard to spot in code review. We want golden-audio tests: render a song
and compare a hash of the output against a committed value. That only works if rendering is
bit-exact across Linux, Windows and macOS, x86-64 and ARM, and debug and release builds.

Two things usually break this:

- **Math library functions.** `sin`, `exp`, `powf` and friends come from each platform's libm
  and are not required to be correctly rounded, so their last bits differ between systems.
- **Accumulated timing.** Adding a fractional "samples per step" over and over drifts and
  depends on evaluation order.

Separately, the audio callback must never block. Allocation can take a lock inside the
allocator, which causes audible dropouts.

## Decision

- DSP and engine code use only IEEE-754 basic operations, which are exactly rounded and
  therefore identical everywhere. Rust does not fuse or reorder float operations, so
  optimization does not change results.
- `soundbit_dsp::math` provides the few functions we need: a range-reduced Taylor `sine`
  (error about 1e-9), equal-tempered `note_frequency` from a table of semitone ratios and exact
  power-of-two scaling, `wrap`, and a rational `soft_clip`.
- FM amplitude and envelope-rate tables are computed in `const fn` at compile time.
- Step positions become sample positions with integer arithmetic:
  `step * sample_rate * 60 / (bpm * steps_per_beat)`, plus a swing offset for odd steps.
- Noise uses NES-style LFSRs with fixed seeds.
- `soundbit-dsp` is `#![no_std]`, so it cannot allocate or do I/O. The engine keeps a fixed
  pool of voices per track with DSP state stored inline. A test runs `Engine::process` under a
  counting global allocator and fails on any allocation.
- The master bus uses a soft clipper (transparent below 0.8) instead of a hard clamp.

## Consequences

- `SHOWCASE_HASH` pins the output of a song that uses every voice kind. CI checks it on all
  three platforms.
- Contributors must not call libm functions in `dsp` or `engine`; `CONTRIBUTING.md` states this rule.
  New math helpers go into `soundbit_dsp::math` with accuracy tests against `std`.
- The FM synthesis approximates the YM2612 for musical use; it is not cycle-accurate.
- Oscillators are PolyBLEP band-limited (pulse, sawtooth). The triangle is deliberately
  stepped (4-bit, NES-like) and the wavetable is not interpolated (Game Boy-like).
