//! A resonant state-variable filter.
//!
//! This is the trapezoidal ("zero-delay feedback") state-variable filter described by Andrew
//! Simper (Cytomic, "Linear Trapezoidal Integrated SVF", 2013). Unlike the classic Chamberlin
//! design it stays stable at every cutoff and resonance, so cutoff sweeps cannot blow up.

use crate::math::sine;

/// Which band the filter outputs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FilterResponse {
    /// Frequencies below the cutoff.
    #[default]
    LowPass,
    /// Frequencies above the cutoff.
    HighPass,
    /// Frequencies around the cutoff.
    BandPass,
}

/// Damping at zero resonance: a Butterworth response (`k = sqrt(2)`), so there is no peak.
const MAX_DAMPING: f64 = core::f64::consts::SQRT_2;

/// Share of the damping that full resonance removes. Below 1, so the filter never
/// self-oscillates into silence-free whistling.
const RESONANCE_RANGE: f64 = 0.96;

/// Highest cutoff as a fraction of the sample rate. The tangent below grows without bound
/// towards Nyquist (0.5).
const MAX_CUTOFF_RATIO: f64 = 0.49;

/// Lowest cutoff in hertz.
const MIN_CUTOFF_HZ: f64 = 10.0;

/// Values smaller than this are flushed to zero, so decaying state does not turn into slow
/// subnormal arithmetic.
const FLUSH_BELOW: f64 = 1e-30;

/// One filter channel with its coefficients and state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Svf {
    response: FilterResponse,
    k: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    ic1eq: f64,
    ic2eq: f64,
}

impl Svf {
    /// Sets the response and resonance (`0` = none, `1` = maximum). Takes effect at the next
    /// [`Svf::set_cutoff`].
    pub fn set_shape(&mut self, response: FilterResponse, resonance: f64) {
        self.response = response;
        self.k = MAX_DAMPING * (1.0 - RESONANCE_RANGE * resonance.clamp(0.0, 1.0));
    }

    /// Sets the cutoff frequency. Cheap enough to call every sample for sweeps.
    pub fn set_cutoff(&mut self, hz: f64, sample_rate: f64) {
        let ratio = if sample_rate > 0.0 {
            (hz.max(MIN_CUTOFF_HZ) / sample_rate).min(MAX_CUTOFF_RATIO)
        } else {
            MAX_CUTOFF_RATIO
        };
        // g = tan(pi * ratio). `sine` takes cycles: pi * ratio radians is ratio / 2 cycles,
        // and the cosine is the sine a quarter cycle later.
        let half = ratio * 0.5;
        let g = sine(half) / sine(half + 0.25);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    /// Clears the filter's memory, e.g. when a new note starts.
    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Filters one sample.
    pub fn process(&mut self, input: f32) -> f32 {
        let v0 = f64::from(input);
        let v3 = v0 - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = flush(2.0 * v1 - self.ic1eq);
        self.ic2eq = flush(2.0 * v2 - self.ic2eq);
        let out = match self.response {
            FilterResponse::LowPass => v2,
            FilterResponse::HighPass => v0 - self.k * v1 - v2,
            FilterResponse::BandPass => v1,
        };
        out as f32
    }
}

fn flush(value: f64) -> f64 {
    if value.abs() < FLUSH_BELOW {
        0.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Oscillator, SineOsc};

    const SAMPLE_RATE: f64 = 48_000.0;

    /// Peak output level for a sine at `hz` once the filter has settled.
    fn gain_at(response: FilterResponse, resonance: f64, cutoff: f64, hz: f64) -> f32 {
        let mut filter = Svf::default();
        filter.set_shape(response, resonance);
        filter.set_cutoff(cutoff, SAMPLE_RATE);
        let mut osc = SineOsc::default();
        osc.set_frequency(hz, SAMPLE_RATE);
        for _ in 0..9_600 {
            filter.process(osc.next_sample());
        }
        (0..4_800)
            .map(|_| filter.process(osc.next_sample()).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn low_pass_keeps_lows_and_cuts_highs() {
        assert!(gain_at(FilterResponse::LowPass, 0.0, 1_000.0, 100.0) > 0.95);
        assert!(gain_at(FilterResponse::LowPass, 0.0, 1_000.0, 10_000.0) < 0.02);
    }

    #[test]
    fn high_pass_keeps_highs_and_cuts_lows() {
        assert!(gain_at(FilterResponse::HighPass, 0.0, 1_000.0, 10_000.0) > 0.95);
        assert!(gain_at(FilterResponse::HighPass, 0.0, 1_000.0, 100.0) < 0.02);
    }

    #[test]
    fn band_pass_peaks_at_the_cutoff() {
        let centre = gain_at(FilterResponse::BandPass, 0.5, 1_000.0, 1_000.0);
        assert!(centre > gain_at(FilterResponse::BandPass, 0.5, 1_000.0, 100.0) * 4.0);
        assert!(centre > gain_at(FilterResponse::BandPass, 0.5, 1_000.0, 10_000.0) * 4.0);
    }

    #[test]
    fn resonance_boosts_the_cutoff() {
        let flat = gain_at(FilterResponse::LowPass, 0.0, 1_000.0, 1_000.0);
        let resonant = gain_at(FilterResponse::LowPass, 1.0, 1_000.0, 1_000.0);
        assert!(resonant > flat * 4.0, "{flat} -> {resonant}");
    }

    #[test]
    fn extreme_settings_stay_finite() {
        let mut filter = Svf::default();
        filter.set_shape(FilterResponse::LowPass, 1.0);
        let mut osc = SineOsc::default();
        osc.set_frequency(3_000.0, SAMPLE_RATE);
        for i in 0..48_000 {
            // Sweep across the whole range, beyond both ends.
            let hz = if i % 2 == 0 { 30_000.0 } else { 1.0 };
            filter.set_cutoff(hz, SAMPLE_RATE);
            let out = filter.process(osc.next_sample());
            assert!(out.is_finite() && out.abs() < 100.0, "sample {i}: {out}");
        }
    }
}
