# Freeverb

- Source: Jezar at Dreampoint, "Freeverb" (2000), original C++ release
  (<https://ccrma.stanford.edu/~jos/pasp/Freeverb.html> describes the algorithm).
- License: public domain (stated by the author in the original release).
- Commit: not applicable (no code was taken from a repository).

## What was taken

The algorithm's structure and published tuning constants: eight parallel lowpass-feedback comb
filters followed by four series allpass filters per channel; comb lengths 1116, 1188, 1277,
1356, 1422, 1491, 1557 and 1617 samples and allpass lengths 556, 441, 341 and 225 samples at
44.1 kHz; a stereo spread of 23 samples; input gain 0.015, room scale 0.28 with offset 0.7,
damping scale 0.4 and allpass feedback 0.5.

`crates/soundbit-dsp/src/reverb.rs` is a reimplementation from that description, adapted to
`no_std` (caller-owned delay buffer), scaled to any sample rate, and with a silence detector
so finished tails can be skipped. No source code was copied.
