//! Band-limited step correction (PolyBLEP).

/// Correction for a unit-height-2 step at phase 0, where `t` is the phase and `dt` the phase
/// increment per sample. Subtracting it from a naive waveform at each discontinuity removes
/// most of the aliasing of pulse and sawtooth waves.
pub(super) fn poly_blep(t: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        0.0
    } else if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}
