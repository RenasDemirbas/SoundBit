//! Linear attack-decay-sustain-release envelopes.

/// Envelope timing in samples for one sample rate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AdsrParams {
    /// Attack length in samples.
    pub attack: u32,
    /// Decay length in samples.
    pub decay: u32,
    /// Sustain level in `[0, 1]`.
    pub sustain: f32,
    /// Release length in samples.
    pub release: u32,
}

impl AdsrParams {
    /// Converts millisecond timings to samples.
    pub fn from_millis(
        attack_ms: u16,
        decay_ms: u16,
        sustain: f32,
        release_ms: u16,
        sample_rate: u32,
    ) -> Self {
        let samples = |ms: u16| {
            let total = u64::from(ms) * u64::from(sample_rate) / 1_000;
            u32::try_from(total).unwrap_or(u32::MAX)
        };
        Self {
            attack: samples(attack_ms),
            decay: samples(decay_ms),
            sustain: sustain.clamp(0.0, 1.0),
            release: samples(release_ms),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Stage {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// An ADSR envelope producing a gain in `[0, 1]` per sample.
///
/// Retriggering starts the attack from the current level, so repeated notes don't click.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Adsr {
    params: AdsrParams,
    stage: Stage,
    level: f32,
    step: f32,
}

impl Adsr {
    /// Changes the timing; takes effect at the next stage change.
    pub fn set_params(&mut self, params: AdsrParams) {
        self.params = params;
    }

    /// Starts the attack stage.
    pub fn note_on(&mut self) {
        if self.params.attack == 0 {
            self.level = 1.0;
            self.enter_decay();
        } else {
            self.stage = Stage::Attack;
            self.step = (1.0 - self.level) / self.params.attack as f32;
        }
    }

    /// Starts the release stage from the current level.
    pub fn note_off(&mut self) {
        if self.stage == Stage::Idle {
            return;
        }
        if self.params.release == 0 {
            self.level = 0.0;
            self.stage = Stage::Idle;
        } else {
            self.stage = Stage::Release;
            self.step = self.level / self.params.release as f32;
        }
    }

    /// Whether the envelope has finished releasing.
    pub fn is_idle(&self) -> bool {
        self.stage == Stage::Idle
    }

    /// Returns the current level and advances by one sample.
    pub fn next_level(&mut self) -> f32 {
        let current = self.level;
        match self.stage {
            Stage::Idle | Stage::Sustain => {}
            Stage::Attack => {
                self.level += self.step;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.enter_decay();
                }
            }
            Stage::Decay => {
                self.level -= self.step;
                if self.level <= self.params.sustain {
                    self.level = self.params.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Release => {
                self.level -= self.step;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        current
    }

    fn enter_decay(&mut self) {
        if self.params.decay == 0 {
            self.level = self.params.sustain;
            self.stage = Stage::Sustain;
        } else {
            self.stage = Stage::Decay;
            self.step = (1.0 - self.params.sustain) / self.params.decay as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(attack: u32, decay: u32, sustain: f32, release: u32) -> Adsr {
        let mut adsr = Adsr::default();
        adsr.set_params(AdsrParams {
            attack,
            decay,
            sustain,
            release,
        });
        adsr
    }

    #[test]
    fn converts_millis_to_samples() {
        let params = AdsrParams::from_millis(10, 20, 0.5, 1_000, 48_000);
        assert_eq!(
            (params.attack, params.decay, params.release),
            (480, 960, 48_000)
        );
    }

    #[test]
    fn walks_through_all_stages() {
        let mut adsr = envelope(4, 4, 0.5, 4);
        adsr.note_on();
        let attack: std::vec::Vec<f32> = (0..5).map(|_| adsr.next_level()).collect();
        assert_eq!(attack, [0.0, 0.25, 0.5, 0.75, 1.0]);
        for _ in 0..4 {
            adsr.next_level();
        }
        assert_eq!(adsr.next_level(), 0.5);
        adsr.note_off();
        for _ in 0..4 {
            adsr.next_level();
        }
        assert!(adsr.is_idle());
        assert_eq!(adsr.next_level(), 0.0);
    }

    #[test]
    fn zero_times_jump_immediately() {
        let mut adsr = envelope(0, 0, 0.8, 0);
        adsr.note_on();
        assert_eq!(adsr.next_level(), 0.8);
        adsr.note_off();
        assert!(adsr.is_idle());
    }

    #[test]
    fn note_off_while_idle_stays_idle() {
        let mut adsr = envelope(10, 10, 1.0, 10);
        adsr.note_off();
        assert!(adsr.is_idle());
    }
}
