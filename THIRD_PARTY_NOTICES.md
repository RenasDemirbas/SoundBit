# Third-party notices

SoundBit's own source code is licensed under the GNU GPL, version 3 or later (see `LICENSE`).
This file lists outside work that SoundBit's code or data is based on. Dependencies pulled in
through Cargo and npm keep their own licenses; `cargo deny` checks that they are compatible.

## Algorithms reimplemented from published descriptions

No code was copied for these; details are in `docs/references/`.

| Work | Author | License | Used in |
|---|---|---|---|
| Freeverb (structure and tuning constants) | Jezar at Dreampoint | Public domain | `crates/soundbit-dsp/src/reverb.rs` |
| Linear trapezoidal state-variable filter | Andrew Simper (Cytomic) | Published paper | `crates/soundbit-dsp/src/filter.rs` |

## Inspiration

SoundBit's simple song model (one-bar patterns placed on an arrangement) is inspired by
[Bosca Ceoil](https://github.com/TerryCavanagh/boscaceoil) by Terry Cavanagh. No code or data was
taken from it.

## Presets and example songs

All presets in `assets/presets/` and all songs in `examples/songs/` are original work by SoundBit
contributors, released under the project license.
