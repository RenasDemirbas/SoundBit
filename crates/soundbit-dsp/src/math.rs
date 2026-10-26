//! Deterministic math helpers.
//!
//! Only IEEE-754 basic operations (`+ - * /`) and conversions are used. They are exactly
//! rounded, so results are bit-identical on every platform. Library functions such as `sin`,
//! `exp` or `powf` are not: their last bits differ between operating systems, which would break
//! golden-audio tests. Use the functions here instead.

/// Frequency of A4 (MIDI note 69) in hertz.
pub const A4_HZ: f64 = 440.0;

/// MIDI note number of A4.
const A4_NOTE: i32 = 69;

/// Frequency ratios of the twelve semitones of an equal-tempered octave: `2^(k/12)`.
const SEMITONE_RATIOS: [f64; 12] = [
    1.0,
    1.059_463_094_359_295_3,
    1.122_462_048_309_373,
    1.189_207_115_002_721,
    1.259_921_049_894_873_2,
    1.334_839_854_170_034_4,
    core::f64::consts::SQRT_2,
    1.498_307_076_876_681_5,
    1.587_401_051_968_199_4,
    1.681_792_830_507_429,
    1.781_797_436_280_678_5,
    1.887_748_625_363_386_8,
];

/// Wraps a phase measured in cycles into `[0, 1)`.
pub fn wrap(phase: f64) -> f64 {
    let fraction = phase - (phase as i64) as f64;
    let wrapped = if fraction < 0.0 {
        fraction + 1.0
    } else {
        fraction
    };
    // A tiny negative fraction can round up to exactly 1.0.
    if wrapped >= 1.0 { 0.0 } else { wrapped }
}

/// Sine of a phase measured in cycles (`sine(0.25) == 1.0`).
///
/// Accurate to about 1e-9, using range reduction and a Taylor polynomial.
pub fn sine(phase: f64) -> f64 {
    // Reduce to t in [-0.5, 0.5), then fold into [-0.25, 0.25] where the series converges fast.
    let mut t = wrap(phase);
    if t >= 0.5 {
        t -= 1.0;
    }
    if t > 0.25 {
        t = 0.5 - t;
    } else if t < -0.25 {
        t = -0.5 - t;
    }
    let x = t * core::f64::consts::TAU;
    let x2 = x * x;
    x * (1.0
        + x2 * (-1.0 / 6.0
            + x2 * (1.0 / 120.0
                + x2 * (-1.0 / 5_040.0
                    + x2 * (1.0 / 362_880.0
                        + x2 * (-1.0 / 39_916_800.0 + x2 * (1.0 / 6_227_020_800.0)))))))
}

/// Equal-tempered frequency of a MIDI note in hertz (A4 = 440 Hz).
pub fn note_frequency(note: u8) -> f64 {
    let semitones = i32::from(note) - A4_NOTE;
    let octaves = semitones.div_euclid(12);
    let ratio = SEMITONE_RATIOS[semitones.rem_euclid(12) as usize];
    // Scaling by powers of two is exact in binary floating point.
    let mut hz = A4_HZ * ratio;
    for _ in 0..octaves.unsigned_abs() {
        hz = if octaves > 0 { hz * 2.0 } else { hz * 0.5 };
    }
    hz
}

/// Taylor coefficients `1/k!` of `e^y` for `k = 0..=12`.
const EXP_SERIES: [f64; 13] = [
    1.0,
    1.0,
    1.0 / 2.0,
    1.0 / 6.0,
    1.0 / 24.0,
    1.0 / 120.0,
    1.0 / 720.0,
    1.0 / 5_040.0,
    1.0 / 40_320.0,
    1.0 / 362_880.0,
    1.0 / 3_628_800.0,
    1.0 / 39_916_800.0,
    1.0 / 479_001_600.0,
];

/// Exponents beyond this are clamped by [`exp2`]; audio never needs more than ±64 octaves.
const EXP2_LIMIT: f64 = 64.0;

/// `2^x`, accurate to about 1e-12 relative error.
///
/// The integer part of `x` is applied exactly through the float's exponent bits, the
/// fractional part with a Taylor series of `e^(f ln 2)`. `x` is clamped to ±64.
pub fn exp2(x: f64) -> f64 {
    let x = x.clamp(-EXP2_LIMIT, EXP2_LIMIT);
    let mut whole = x as i64;
    if (whole as f64) > x {
        whole -= 1;
    }
    let y = (x - whole as f64) * core::f64::consts::LN_2;
    // Horner's scheme from the highest power down: 1 + y(1 + y(1/2 + y(1/6 + ...))).
    let fraction = EXP_SERIES
        .iter()
        .rev()
        .fold(0.0, |sum, &coefficient| coefficient + y * sum);
    // A biased exponent between 959 and 1087 is always a normal float.
    let scale = f64::from_bits(((whole + 1_023) as u64) << 52);
    fraction * scale
}

/// Frequency ratio of an interval in semitones (12 = one octave up, -12 = one octave down).
pub fn semitone_ratio(semitones: f64) -> f64 {
    exp2(semitones / 12.0)
}

/// Level above which [`soft_clip`] starts to compress.
const SOFT_CLIP_KNEE: f32 = 0.8;

/// Keeps a sample inside `(-1, 1)` without hard clipping: unchanged up to the knee, then
/// compressed smoothly (with continuous slope) towards full scale.
pub fn soft_clip(sample: f32) -> f32 {
    let magnitude = sample.abs();
    if magnitude <= SOFT_CLIP_KNEE {
        return sample;
    }
    let headroom = 1.0 - SOFT_CLIP_KNEE;
    let over = (magnitude - SOFT_CLIP_KNEE) / headroom;
    let limited = SOFT_CLIP_KNEE + headroom * over / (1.0 + over);
    if sample < 0.0 { -limited } else { limited }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() <= tolerance
    }

    #[test]
    fn wrap_stays_in_unit_interval() {
        for value in [-2.75, -1.0, -0.25, 0.0, 0.5, 0.999, 1.0, 3.25, -1e-18] {
            let wrapped = wrap(value);
            assert!((0.0..1.0).contains(&wrapped), "{value} -> {wrapped}");
        }
        assert!(close(wrap(-0.25), 0.75, 1e-12));
        assert!(close(wrap(3.25), 0.25, 1e-12));
    }

    #[test]
    fn sine_matches_std_closely() {
        for i in -2000..=2000 {
            let phase = f64::from(i) / 997.0;
            let expected = (phase * core::f64::consts::TAU).sin();
            assert!(close(sine(phase), expected, 2e-9), "phase {phase}");
        }
        assert!(close(sine(0.25), 1.0, 1e-9));
        assert!(close(sine(0.5), 0.0, 1e-9));
    }

    #[test]
    fn soft_clip_is_transparent_below_the_knee() {
        for sample in [-0.8, -0.5, 0.0, 0.3, 0.8] {
            assert_eq!(soft_clip(sample), sample);
        }
    }

    #[test]
    fn soft_clip_is_bounded_monotonic_and_odd() {
        let mut previous = soft_clip(0.0);
        for i in 1..=10_000 {
            let sample = i as f32 / 1_000.0;
            let clipped = soft_clip(sample);
            assert!(
                clipped < 1.0 && clipped >= previous,
                "{sample} -> {clipped}"
            );
            assert_eq!(soft_clip(-sample), -clipped);
            previous = clipped;
        }
    }

    #[test]
    fn exp2_matches_std_closely() {
        for i in -4_000..=4_000 {
            let x = f64::from(i) / 97.0;
            let expected = x.exp2();
            assert!(
                close(exp2(x), expected, expected * 1e-10),
                "2^{x}: {} vs {expected}",
                exp2(x)
            );
        }
        assert_eq!(exp2(0.0), 1.0);
        assert_eq!(exp2(3.0), 8.0);
        assert_eq!(exp2(-1.0), 0.5);
    }

    #[test]
    fn exp2_clamps_extremes() {
        assert_eq!(exp2(1_000.0), exp2(64.0));
        assert_eq!(exp2(-1_000.0), exp2(-64.0));
        assert!(exp2(-64.0) > 0.0);
    }

    #[test]
    fn semitone_ratios_match_equal_temperament() {
        assert!(close(semitone_ratio(12.0), 2.0, 1e-12));
        assert!(close(semitone_ratio(-24.0), 0.25, 1e-12));
        assert!(close(semitone_ratio(7.0), SEMITONE_RATIOS[7], 1e-9));
    }

    #[test]
    fn reference_frequencies() {
        assert!(close(note_frequency(69), 440.0, 1e-9));
        assert!(close(note_frequency(57), 220.0, 1e-9));
        assert!(close(note_frequency(81), 880.0, 1e-9));
        assert!(close(note_frequency(60), 261.625_565_300_598_6, 1e-9));
        assert!(close(note_frequency(0), 8.175_798_915_643_707, 1e-9));
        assert!(close(note_frequency(127), 12_543.853_951_415_975, 1e-6));
    }
}
