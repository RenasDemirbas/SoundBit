//! Lookup tables for FM synthesis, computed at compile time so they are identical everywhere.

/// Envelope attenuation is measured in steps of 0.09375 dB, as on the YM2612.
pub(super) const ATTENUATION_STEPS: usize = 1024;

/// The quietest attenuation; treated as silence.
pub(super) const MAX_ATTENUATION: f32 = (ATTENUATION_STEPS - 1) as f32;

/// Attenuation steps per total-level unit (0.75 dB).
pub(super) const TOTAL_LEVEL_STEP: f32 = 8.0;

/// Attenuation steps per sustain-level unit (3 dB).
pub(super) const SUSTAIN_LEVEL_STEP: f32 = 32.0;

/// Sustain level 15 jumps to 93 dB instead of 45 dB, as on the YM2612.
pub(super) const SUSTAIN_LEVEL_MAX: f32 = 992.0;

/// `10^(-0.09375 / 20)`: the amplitude ratio of one attenuation step.
const STEP_RATIO: f64 = 0.989_264_671_767_740_9;

/// Linear amplitude for each attenuation step. The last entry is exactly silent.
pub(super) static AMPLITUDE: [f32; ATTENUATION_STEPS] = amplitude_table();

const fn amplitude_table() -> [f32; ATTENUATION_STEPS] {
    let mut table = [0.0; ATTENUATION_STEPS];
    let mut amplitude = 1.0_f64;
    let mut index = 0;
    while index < ATTENUATION_STEPS - 1 {
        table[index] = amplitude as f32;
        amplitude *= STEP_RATIO;
        index += 1;
    }
    table
}

/// `sqrt(0.5)`: each envelope rate step halves the stage time every two steps.
const SQRT_HALF: f64 = core::f64::consts::FRAC_1_SQRT_2;

/// Time a rate-1 envelope stage needs to sweep the full attenuation range.
const SLOWEST_STAGE_SECONDS: f64 = 10.0;

/// Seconds to sweep the full attenuation range for each envelope rate (0–31).
/// Rate 0 means "never" and is handled separately.
pub(super) const RATE_SECONDS: [f64; 32] = rate_table();

const fn rate_table() -> [f64; 32] {
    let mut table = [0.0; 32];
    let mut seconds = SLOWEST_STAGE_SECONDS / SQRT_HALF;
    let mut rate = 0;
    while rate < 32 {
        table[rate] = seconds;
        seconds *= SQRT_HALF;
        rate += 1;
    }
    table
}

/// Operator 1 self-feedback depth in cycles for each feedback level (0–7).
pub(super) const FEEDBACK_DEPTH: [f64; 8] = [
    0.0,
    1.0 / 32.0,
    1.0 / 16.0,
    1.0 / 8.0,
    1.0 / 4.0,
    1.0 / 2.0,
    1.0,
    2.0,
];

/// Phase modulation, in cycles, applied by a modulator at full level.
pub(super) const MODULATION_DEPTH: f64 = 1.0;

/// Frequency change per detune step (about 1.4 cents).
pub(super) const DETUNE_STEP: f64 = 0.000_8;

/// Which operators are carriers (heard directly) for each algorithm 0–7.
pub(super) const CARRIERS: [[bool; 4]; 8] = [
    [false, false, false, true],
    [false, false, false, true],
    [false, false, false, true],
    [false, false, false, true],
    [false, true, false, true],
    [false, true, true, true],
    [false, true, true, true],
    [true, true, true, true],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplitude_falls_monotonically_to_silence() {
        assert_eq!(AMPLITUDE[0], 1.0);
        assert!(AMPLITUDE.windows(2).all(|pair| pair[0] >= pair[1]));
        assert_eq!(AMPLITUDE[ATTENUATION_STEPS - 1], 0.0);
        // 64 steps is 6 dB, roughly half the amplitude.
        assert!((AMPLITUDE[64] - 0.501).abs() < 0.001);
    }

    #[test]
    fn rates_speed_up() {
        assert!(RATE_SECONDS.windows(2).all(|pair| pair[0] > pair[1]));
        assert!((RATE_SECONDS[1] - SLOWEST_STAGE_SECONDS).abs() < 1e-9);
        const { assert!(RATE_SECONDS[31] < 0.001) };
    }
}
