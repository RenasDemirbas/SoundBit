//! Four-operator FM synthesis in the style of the Yamaha YM2612 (Sega Genesis / Mega Drive).
//!
//! This is an approximation for musical use, not a cycle-accurate emulation. Operators are
//! numbered 1–4 and routed by one of eight algorithms:
//!
//! | Algorithm | Routing                     | Carriers   |
//! |-----------|-----------------------------|------------|
//! | 0         | 1 → 2 → 3 → 4               | 4          |
//! | 1         | (1 + 2) → 3 → 4             | 4          |
//! | 2         | (1 + (2 → 3)) → 4           | 4          |
//! | 3         | ((1 → 2) + 3) → 4           | 4          |
//! | 4         | (1 → 2) + (3 → 4)           | 2, 4       |
//! | 5         | 1 → (2, 3, 4)               | 2, 3, 4    |
//! | 6         | (1 → 2) + 3 + 4             | 2, 3, 4    |
//! | 7         | 1 + 2 + 3 + 4               | all        |
//!
//! Operator 1 can modulate itself (feedback). Carrier outputs are averaged so every algorithm
//! has a similar peak level.

mod envelope;
mod operator;
mod tables;

pub use operator::{OperatorParams, YmOperator};

use operator::Operator;
use tables::{CARRIERS, FEEDBACK_DEPTH, MODULATION_DEPTH};

/// A complete FM patch converted for one sample rate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FmParams {
    algorithm: usize,
    feedback: f64,
    operators: [OperatorParams; 4],
}

impl FmParams {
    /// Converts raw parameters: `algorithm` 0–7, `feedback` 0–7. Out-of-range values are
    /// clamped.
    pub fn from_ym(
        algorithm: u8,
        feedback: u8,
        operators: &[YmOperator; 4],
        sample_rate: f64,
    ) -> Self {
        Self {
            algorithm: usize::from(algorithm.min(7)),
            feedback: FEEDBACK_DEPTH[usize::from(feedback.min(7))],
            operators: operators.map(|op| OperatorParams::from_ym(&op, sample_rate)),
        }
    }
}

/// One playing FM voice.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FmSynth {
    algorithm: usize,
    feedback: f64,
    operators: [Operator; 4],
    history: [f64; 2],
}

impl FmSynth {
    /// Starts a note at `hz` with all operator envelopes in attack.
    pub fn note_on(&mut self, params: &FmParams, hz: f64, sample_rate: f64) {
        self.algorithm = params.algorithm;
        self.feedback = params.feedback;
        self.history = [0.0; 2];
        for (operator, op_params) in self.operators.iter_mut().zip(params.operators) {
            operator.key_on(op_params, hz, sample_rate);
        }
    }

    /// Changes the pitch of the playing note without restarting it, e.g. for vibrato.
    pub fn set_frequency(&mut self, hz: f64, sample_rate: f64) {
        for operator in &mut self.operators {
            operator.set_frequency(hz, sample_rate);
        }
    }

    /// Releases all operators.
    pub fn note_off(&mut self) {
        for operator in &mut self.operators {
            operator.key_off();
        }
    }

    /// Whether every carrier has finished releasing, so the voice is inaudible.
    pub fn is_silent(&self) -> bool {
        CARRIERS[self.algorithm]
            .iter()
            .zip(&self.operators)
            .all(|(&carrier, operator)| !carrier || operator.is_off())
    }

    /// Produces the next sample, in `[-1, 1]`.
    pub fn next_sample(&mut self) -> f32 {
        let feedback = f64::midpoint(self.history[0], self.history[1]) * self.feedback;
        let [op1, op2, op3, op4] = &mut self.operators;
        let out1 = op1.output(feedback);
        self.history = [self.history[1], out1];

        let depth = MODULATION_DEPTH;
        let mixed = match self.algorithm {
            0 => {
                let out2 = op2.output(out1 * depth);
                let out3 = op3.output(out2 * depth);
                op4.output(out3 * depth)
            }
            1 => {
                let out2 = op2.output(0.0);
                let out3 = op3.output((out1 + out2) * depth);
                op4.output(out3 * depth)
            }
            2 => {
                let out2 = op2.output(0.0);
                let out3 = op3.output(out2 * depth);
                op4.output((out1 + out3) * depth)
            }
            3 => {
                let out2 = op2.output(out1 * depth);
                let out3 = op3.output(0.0);
                op4.output((out2 + out3) * depth)
            }
            4 => {
                let out2 = op2.output(out1 * depth);
                let out3 = op3.output(0.0);
                f64::midpoint(out2, op4.output(out3 * depth))
            }
            5 => {
                let modulation = out1 * depth;
                (op2.output(modulation) + op3.output(modulation) + op4.output(modulation)) / 3.0
            }
            6 => (op2.output(out1 * depth) + op3.output(0.0) + op4.output(0.0)) / 3.0,
            _ => (out1 + op2.output(0.0) + op3.output(0.0) + op4.output(0.0)) / 4.0,
        };
        mixed.clamp(-1.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 48_000.0;

    fn carrier() -> YmOperator {
        YmOperator {
            multiple: 1,
            attack_rate: 31,
            release_rate: 15,
            ..YmOperator::default()
        }
    }

    fn silent() -> YmOperator {
        YmOperator {
            total_level: 127,
            ..carrier()
        }
    }

    fn peak(synth: &mut FmSynth, samples: usize) -> f32 {
        (0..samples)
            .map(|_| synth.next_sample().abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn single_carrier_is_a_sine() {
        let params = FmParams::from_ym(
            7,
            0,
            &[carrier(), silent(), silent(), silent()],
            SAMPLE_RATE,
        );
        let mut synth = FmSynth::default();
        synth.note_on(&params, 440.0, SAMPLE_RATE);
        let level = peak(&mut synth, 4_800);
        // Algorithm 7 averages four operators, so one full-level carrier peaks at 1/4.
        assert!((level - 0.25).abs() < 0.01, "peak {level}");
    }

    #[test]
    fn every_algorithm_is_bounded_and_releases() {
        for algorithm in 0..8 {
            let params = FmParams::from_ym(algorithm, 7, &[carrier(); 4], SAMPLE_RATE);
            let mut synth = FmSynth::default();
            synth.note_on(&params, 220.0, SAMPLE_RATE);
            assert!(
                peak(&mut synth, 2_400) > 0.0,
                "algorithm {algorithm} is silent"
            );
            assert!(!synth.is_silent());
            synth.note_off();
            peak(&mut synth, 48_000);
            assert!(synth.is_silent(), "algorithm {algorithm} never released");
        }
    }

    #[test]
    fn idle_synth_is_silent() {
        let mut synth = FmSynth::default();
        assert!(synth.is_silent());
        assert_eq!(synth.next_sample(), 0.0);
    }
}
