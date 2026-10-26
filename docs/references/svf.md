# Trapezoidal state-variable filter

- Source: Andrew Simper, "Linear Trapezoidal Integrated SVF" (Cytomic technical paper, 2013),
  <https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf>.
- License: a published technical paper describing mathematics; no code or data was copied.
- Commit: not applicable.

## What was taken

The filter equations (coefficients `g = tan(pi * fc / fs)`, `k = 1 / Q`, `a1`, `a2`, `a3` and
the two integrator states). `crates/soundbit-dsp/src/filter.rs` implements them in its own
code, computing the tangent from the deterministic `math::sine` so results are bit-identical
on every platform (ADR 0003).
